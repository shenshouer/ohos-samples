//! XComponent window facade. Scene painting is introduced in M3-2.

use std::cell::{Cell, RefCell};
use std::ptr::NonNull;
use std::rc::Rc;
use std::sync::Arc;

use crate::gpui_core::*;
use futures::channel::oneshot;
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, OhosNdkWindowHandle,
    RawWindowHandle, WindowHandle,
};

use crate::host::surface::{self, SurfaceSnapshot};
use crate::sdk_ffi::display;

#[derive(Default)]
struct Callbacks {
    request_frame: Option<Box<dyn FnMut(RequestFrameOptions)>>,
    input: Option<Box<dyn FnMut(PlatformInput) -> DispatchEventResult>>,
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

struct EmptyAtlas;
impl PlatformAtlas for EmptyAtlas {
    fn get_or_insert_with<'a>(
        &self,
        _key: AtlasKey,
        _build: &mut dyn FnMut()
            -> Result<Option<(Size<DevicePixels>, std::borrow::Cow<'a, [u8]>)>>,
    ) -> Result<Option<AtlasTile>> {
        Err(crate::gpui_core::private::anyhow::anyhow!(
            "atlas pending M3-2"
        ))
    }
    fn remove(&self, _key: &AtlasKey) {}
}

pub(crate) struct OhosWindow {
    snapshot: SurfaceSnapshot,
    display: Rc<dyn PlatformDisplay>,
    size: Cell<Size<Pixels>>,
    input_handler: RefCell<Option<PlatformInputHandler>>,
    callbacks: RefCell<Callbacks>,
    atlas: Arc<EmptyAtlas>,
}

impl OhosWindow {
    pub(crate) fn new(snapshot: SurfaceSnapshot, display: Rc<dyn PlatformDisplay>) -> Self {
        Self {
            snapshot,
            display,
            size: Cell::new({
                let (width, height) = snapshot.logical_dimensions();
                size(px(width), px(height))
            }),
            input_handler: RefCell::new(None),
            callbacks: RefCell::new(Callbacks::default()),
            atlas: Arc::new(EmptyAtlas),
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
    fn draw(&self, _scene: &Scene) {}
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
