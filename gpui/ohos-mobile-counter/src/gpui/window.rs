//! XComponent window facade and SDK frame callback delivery.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::ptr::NonNull;
use std::rc::{Rc, Weak};
use std::sync::Arc;

use crate::gpui_core::*;
use futures::channel::oneshot;
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, OhosNdkWindowHandle,
    RawWindowHandle, WindowHandle,
};

use crate::host::frame;
use crate::host::logging::log;
use crate::host::surface::{self, SurfaceSnapshot};
use crate::sdk_ffi::{display, hilog};

use super::atlas::MonoAtlas;

thread_local! {
    static FRAME_WINDOWS: RefCell<Vec<(SurfaceSnapshot, Weak<RefCell<Callbacks>>)>> = const { RefCell::new(Vec::new()) };
}

pub(crate) fn pump_frames() {
    let windows = FRAME_WINDOWS.with(|slot| {
        let mut windows = slot.borrow_mut();
        windows.retain(|(_, callbacks)| callbacks.strong_count() > 0);
        windows.clone()
    });
    for (snapshot, callbacks) in windows {
        let current = surface::snapshot().is_some_and(|current| {
            current.window == snapshot.window && current.generation == snapshot.generation
        });
        if current && let Some(callbacks) = callbacks.upgrade() {
            if let Some(callback) = callbacks.borrow_mut().request_frame.as_mut() {
                callback(RequestFrameOptions {
                    signal_source: FrameRequestSource::LocalSchedule,
                    ..Default::default()
                });
            }
        }
    }
}

/// SDK coordinates are physical pixels relative to the XComponent Surface.
#[derive(Clone, Copy)]
pub(crate) struct TouchInput {
    pub(crate) window: usize,
    pub(crate) generation: u64,
    pub(crate) source_id: i32,
    pub(crate) phase: TouchPhase,
    pub(crate) x_pixels: f32,
    pub(crate) y_pixels: f32,
    pub(crate) force: Option<f32>,
}

#[derive(Default)]
struct TouchIds {
    next: u64,
    active: HashMap<i32, TouchId>,
}

impl TouchIds {
    fn for_phase(&mut self, source_id: i32, phase: TouchPhase) -> Option<TouchId> {
        match phase {
            TouchPhase::Started => {
                if self.active.contains_key(&source_id) {
                    return None;
                }
                self.next = self.next.checked_add(1)?;
                let id = TouchId(self.next);
                self.active.insert(source_id, id);
                Some(id)
            }
            TouchPhase::Moved => self.active.get(&source_id).copied(),
            TouchPhase::Ended | TouchPhase::Cancelled => self.active.remove(&source_id),
        }
    }
}

/// Called only by the ArkUI pump, after the SDK callback has returned.
pub(crate) fn dispatch_touch(input: TouchInput) -> bool {
    let Some(snapshot) = surface::snapshot() else {
        return false;
    };
    if snapshot.window != input.window || snapshot.generation != input.generation {
        return false;
    }
    let callbacks = FRAME_WINDOWS.with(|slot| {
        slot.borrow()
            .iter()
            .find(|(window, _)| {
                window.window == input.window && window.generation == input.generation
            })
            .and_then(|(_, callbacks)| callbacks.upgrade())
    });
    let Some(callbacks) = callbacks else {
        return false;
    };
    let density = display::default_display()
        .map(|info| info.density)
        .unwrap_or(1.0);
    if !density.is_finite() || density <= 0.0 {
        return false;
    }
    let (id, mut callback) = {
        let mut state = callbacks.borrow_mut();
        if state.input.is_none() {
            return false;
        }
        let Some(id) = state.touch_ids.for_phase(input.source_id, input.phase) else {
            log(
                hilog::LOG_WARN,
                format!(
                    "GPUI touch ignored: unmatched or duplicate phase={:?} source_id={}",
                    input.phase, input.source_id
                ),
            );
            return false;
        };
        (id, state.input.take().expect("input callback checked"))
    };
    let x = input.x_pixels / density;
    let y = input.y_pixels / density;
    callback(PlatformInput::Touch(TouchEvent {
        id,
        phase: input.phase,
        position: point(px(x), px(y)),
        predicted_position: None,
        force: input.force,
    }));
    let mut state = callbacks.borrow_mut();
    if state.input.is_none() {
        state.input = Some(callback);
    }
    if input.phase != TouchPhase::Moved {
        log(
            hilog::LOG_INFO,
            format!(
                "GPUI touch dispatched generation={} id={} phase={:?} px=({:.1},{:.1}) logical=({:.1},{:.1}) density={:.2}",
                input.generation, id.0, input.phase, input.x_pixels, input.y_pixels, x, y, density
            ),
        );
    }
    true
}

#[derive(Default)]
struct Callbacks {
    request_frame: Option<Box<dyn FnMut(RequestFrameOptions)>>,
    input: Option<Box<dyn FnMut(PlatformInput) -> DispatchEventResult>>,
    touch_ids: TouchIds,
    active: Option<Box<dyn FnMut(bool)>>,
    visibility: Option<Box<dyn FnMut(WindowVisibility)>>,
    hover: Option<Box<dyn FnMut(bool)>>,
    resize: Option<Box<dyn FnMut(Size<Pixels>, f32)>>,
    moved: Option<Box<dyn FnMut()>>,
    display: Option<Box<dyn FnMut()>>,
    should_close: Option<Box<dyn FnMut() -> bool>>,
    hit_test: Option<Box<dyn FnMut() -> Option<WindowControlArea>>>,
    close: Option<Box<dyn FnOnce()>>,
    appearance: Option<Box<dyn FnMut()>>,
}

pub(crate) struct OhosWindow {
    snapshot: SurfaceSnapshot,
    display: Rc<dyn PlatformDisplay>,
    size: Cell<Size<Pixels>>,
    input_handler: RefCell<Option<PlatformInputHandler>>,
    callbacks: Rc<RefCell<Callbacks>>,
    atlas: Arc<MonoAtlas>,
}

impl OhosWindow {
    pub(crate) fn new(snapshot: SurfaceSnapshot, display: Rc<dyn PlatformDisplay>) -> Self {
        let callbacks = Rc::new(RefCell::new(Callbacks::default()));
        FRAME_WINDOWS.with(|slot| {
            slot.borrow_mut()
                .push((snapshot, Rc::downgrade(&callbacks)));
        });
        Self {
            snapshot,
            display,
            size: Cell::new({
                let (width, height) = snapshot.logical_dimensions();
                size(px(width), px(height))
            }),
            input_handler: RefCell::new(None),
            callbacks,
            atlas: Arc::new(MonoAtlas::default()),
        }
    }

    fn is_current(&self) -> bool {
        surface::snapshot().is_some_and(|s| {
            s.window == self.snapshot.window && s.generation == self.snapshot.generation
        })
    }
}

impl HasWindowHandle for OhosWindow {
    fn window_handle(&self) -> std::result::Result<WindowHandle<'_>, HandleError> {
        if !self.is_current() {
            return Err(HandleError::Unavailable);
        }
        let pointer = NonNull::new(self.snapshot.window as *mut std::ffi::c_void)
            .ok_or(HandleError::Unavailable)?;
        let raw = RawWindowHandle::OhosNdk(OhosNdkWindowHandle::new(pointer));
        // SAFETY: XComponent owns this OHNativeWindow, and the borrowed handle
        // cannot outlive &self; stale generations are rejected above.
        Ok(unsafe { WindowHandle::borrow_raw(raw) })
    }
}

impl HasDisplayHandle for OhosWindow {
    fn display_handle(&self) -> std::result::Result<DisplayHandle<'_>, HandleError> {
        Ok(DisplayHandle::ohos())
    }
}

impl PlatformWindow for OhosWindow {
    fn bounds(&self) -> Bounds<Pixels> {
        Bounds::new(point(px(0.0), px(0.0)), self.size.get())
    }
    fn is_maximized(&self) -> bool {
        false
    }
    fn window_bounds(&self) -> WindowBounds {
        WindowBounds::Windowed(self.bounds())
    }
    fn content_size(&self) -> Size<Pixels> {
        self.size.get()
    }
    fn resize(&mut self, size: Size<Pixels>) {
        self.size.set(size);
    }
    fn scale_factor(&self) -> f32 {
        display::default_display()
            .map(|info| info.density)
            .unwrap_or(1.0)
    }
    fn appearance(&self) -> WindowAppearance {
        WindowAppearance::Light
    }
    fn display(&self) -> Option<Rc<dyn PlatformDisplay>> {
        Some(self.display.clone())
    }
    fn mouse_position(&self) -> Point<Pixels> {
        point(px(0.0), px(0.0))
    }
    fn modifiers(&self) -> Modifiers {
        Modifiers::default()
    }
    fn capslock(&self) -> Capslock {
        Capslock::default()
    }
    fn set_input_handler(&mut self, handler: PlatformInputHandler) {
        *self.input_handler.borrow_mut() = Some(handler);
    }
    fn take_input_handler(&mut self) -> Option<PlatformInputHandler> {
        self.input_handler.borrow_mut().take()
    }
    fn prompt(
        &self,
        _level: PromptLevel,
        _msg: &str,
        _detail: Option<&str>,
        _answers: &[PromptButton],
    ) -> Option<oneshot::Receiver<usize>> {
        None
    }
    fn activate(&self) {}
    fn is_active(&self) -> bool {
        self.is_current()
    }
    fn visibility(&self) -> WindowVisibility {
        if self.is_current() {
            WindowVisibility::Visible
        } else {
            WindowVisibility::Hidden
        }
    }
    fn is_hovered(&self) -> bool {
        false
    }
    fn background_appearance(&self) -> WindowBackgroundAppearance {
        WindowBackgroundAppearance::Opaque
    }
    fn set_title(&mut self, _title: &str) {}
    fn set_background_appearance(&self, _background: WindowBackgroundAppearance) {}
    fn minimize(&self) {}
    fn zoom(&self) {}
    fn toggle_fullscreen(&self) {}
    fn is_fullscreen(&self) -> bool {
        false
    }
    fn on_request_frame(&self, callback: Box<dyn FnMut(RequestFrameOptions)>) {
        self.callbacks.borrow_mut().request_frame = Some(callback);
    }
    fn frame_waker(&self) -> Option<Rc<dyn Fn()>> {
        let generation = self.snapshot.generation;
        Some(Rc::new(move || frame::wake_for(generation)))
    }
    fn schedule_frame(&self) {
        frame::wake_for(self.snapshot.generation);
    }
    fn on_input(&self, callback: Box<dyn FnMut(PlatformInput) -> DispatchEventResult>) {
        self.callbacks.borrow_mut().input = Some(callback);
    }
    fn on_active_status_change(&self, callback: Box<dyn FnMut(bool)>) {
        self.callbacks.borrow_mut().active = Some(callback);
    }
    fn on_visibility_change(&self, callback: Box<dyn FnMut(WindowVisibility)>) {
        self.callbacks.borrow_mut().visibility = Some(callback);
    }
    fn on_hover_status_change(&self, callback: Box<dyn FnMut(bool)>) {
        self.callbacks.borrow_mut().hover = Some(callback);
    }
    fn on_resize(&self, callback: Box<dyn FnMut(Size<Pixels>, f32)>) {
        self.callbacks.borrow_mut().resize = Some(callback);
    }
    fn on_moved(&self, callback: Box<dyn FnMut()>) {
        self.callbacks.borrow_mut().moved = Some(callback);
    }
    fn on_display_changed(&self, callback: Box<dyn FnMut()>) {
        self.callbacks.borrow_mut().display = Some(callback);
    }
    fn on_should_close(&self, callback: Box<dyn FnMut() -> bool>) {
        self.callbacks.borrow_mut().should_close = Some(callback);
    }
    fn on_hit_test_window_control(&self, callback: Box<dyn FnMut() -> Option<WindowControlArea>>) {
        self.callbacks.borrow_mut().hit_test = Some(callback);
    }
    fn on_close(&self, callback: Box<dyn FnOnce()>) {
        self.callbacks.borrow_mut().close = Some(callback);
    }
    fn on_appearance_changed(&self, callback: Box<dyn FnMut()>) {
        self.callbacks.borrow_mut().appearance = Some(callback);
    }
    fn draw(&self, scene: &Scene) {
        let batches = scene
            .batches()
            .map(|batch| batch.label())
            .collect::<Vec<_>>();
        log(
            hilog::LOG_INFO,
            format!("GPUI source Scene batches: {}", batches.join(", ")),
        );
        if let Err(error) = surface::draw_scene(self.snapshot, scene, &self.atlas) {
            log(hilog::LOG_ERROR, format!("GPUI Scene draw failed: {error}"));
        }
    }
    fn sprite_atlas(&self) -> Arc<dyn PlatformAtlas> {
        self.atlas.clone()
    }
    fn is_subpixel_rendering_supported(&self) -> bool {
        false
    }
    fn gpu_specs(&self) -> Option<GpuSpecs> {
        None
    }
    fn update_ime_position(&self, _bounds: Bounds<Pixels>) {}
}
