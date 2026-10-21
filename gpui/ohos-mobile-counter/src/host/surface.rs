//! XComponent lifecycle state and synchronous SDK callback adapters.

use std::ffi::c_void;
use std::sync::Mutex;

use crate::gpui::atlas::MonoAtlas;
use crate::gpui::{self, SurfaceEvent};
use crate::gpui_core::Scene;
use crate::host::frame;
use crate::host::logging::log;
use crate::host::touch;
use crate::render::screen::ScreenDrawing;
use crate::sdk_ffi::{drawing, hilog, native_window, xcomponent as xc};

#[derive(Default)]
struct SurfaceState {
    registered_component: usize,
    window: usize,
    generation: u64,
    width: u64,
    height: u64,
    pixel_width: i32,
    pixel_height: i32,
    drawing: Option<ScreenDrawing>,
}

static STATE: Mutex<SurfaceState> = Mutex::new(SurfaceState {
    registered_component: 0,
    window: 0,
    generation: 0,
    width: 0,
    height: 0,
    pixel_width: 0,
    pixel_height: 0,
    drawing: None,
});

#[derive(Clone, Copy)]
pub(crate) struct SurfaceSnapshot {
    pub(crate) window: usize,
    pub(crate) generation: u64,
    pub(crate) pixel_width: i32,
    pub(crate) pixel_height: i32,
}

impl SurfaceSnapshot {
    pub(crate) fn logical_dimensions(self) -> (f32, f32) {
        let density = crate::sdk_ffi::display::default_display()
            .map(|info| info.density)
            .unwrap_or(1.0);
        (
            self.pixel_width as f32 / density,
            self.pixel_height as f32 / density,
        )
    }
}

pub(crate) fn snapshot() -> Option<SurfaceSnapshot> {
    let state = STATE.lock().ok()?;
    (state.window != 0 && state.pixel_width > 0 && state.pixel_height > 0).then_some(
        SurfaceSnapshot {
            window: state.window,
            generation: state.generation,
            pixel_width: state.pixel_width,
            pixel_height: state.pixel_height,
        },
    )
}

pub(crate) fn draw_scene(
    expected: SurfaceSnapshot,
    scene: &Scene,
    atlas: &MonoAtlas,
) -> crate::gpui_core::Result<()> {
    let mut state = STATE
        .lock()
        .map_err(|_| crate::gpui_core::private::anyhow::anyhow!("surface state lock poisoned"))?;
    if state.window != expected.window || state.generation != expected.generation {
        return Err(crate::gpui_core::private::anyhow::anyhow!(
            "scene draw rejected for stale Surface"
        ));
    }
    let drawing = state
        .drawing
        .as_mut()
        .ok_or_else(|| crate::gpui_core::private::anyhow::anyhow!("Drawing surface unavailable"))?;
    let stats = drawing.paint_scene(scene, atlas)?;
    drop(state);
    log(
        hilog::LOG_INFO,
        format!(
            "GPUI Scene flush generation={} quads={} monochrome_glyphs={}",
            expected.generation, stats.quads, stats.glyphs
        ),
    );
    Ok(())
}

// The SDK accepts a pointer to this table. It must outlive every callback.
static mut CALLBACKS: xc::Callbacks = xc::Callbacks {
    on_surface_created: Some(on_created),
    on_surface_changed: Some(on_changed),
    on_surface_destroyed: Some(on_destroyed),
    dispatch_touch_event: Some(on_touch),
};

fn thread() -> String {
    format!("{:?}", std::thread::current().id())
}

/// Called from the XComponent module initialization, possibly more than once.
pub fn register(component: *mut xc::XComponent) {
    if component.is_null() {
        log(hilog::LOG_ERROR, "register: null XComponent".into());
        return;
    }
    let Ok(mut state) = STATE.lock() else {
        log(hilog::LOG_ERROR, "register: state lock poisoned".into());
        return;
    };
    if state.registered_component == component as usize {
        log(
            hilog::LOG_INFO,
            format!(
                "register: duplicate component={component:p} thread={}",
                thread()
            ),
        );
        return;
    }
    // Release the lock before the SDK call: registration may invoke a callback.
    state.drawing = None;
    state.registered_component = component as usize;
    state.window = 0;
    state.width = 0;
    state.height = 0;
    state.pixel_width = 0;
    state.pixel_height = 0;
    drop(state);
    // SAFETY: component comes from napi_unwrap of the SDK-owned XComponent;
    // CALLBACKS is process-lifetime storage.
    let result = unsafe { xc::OH_NativeXComponent_RegisterCallback(component, &raw mut CALLBACKS) };
    if let Err(error) = xc::check_result(result) {
        if let Ok(mut state) = STATE.lock() {
            if state.registered_component == component as usize {
                state.registered_component = 0;
                state.window = 0;
                state.width = 0;
                state.height = 0;
                state.pixel_width = 0;
                state.pixel_height = 0;
                state.drawing = None;
            }
        }
        log(
            hilog::LOG_ERROR,
            format!("register: failed code={} component={component:p}", error.0),
        );
        return;
    }
    log(
        hilog::LOG_INFO,
        format!("register: ok component={component:p} thread={}", thread()),
    );
}

fn measure(component: *mut xc::XComponent, window: *mut c_void) -> Result<(u64, u64), i32> {
    let (mut width, mut height) = (0, 0);
    // SAFETY: the SDK supplies these pointers during the callback; output
    // references are writable for the duration of this synchronous call.
    let result = unsafe {
        xc::OH_NativeXComponent_GetXComponentSize(component, window, &mut width, &mut height)
    };
    xc::check_result(result)
        .map(|()| (width, height))
        .map_err(|error| error.0)
}

fn update_surface(kind: &str, component: *mut xc::XComponent, window: *mut c_void) {
    if component.is_null() || window.is_null() {
        log(hilog::LOG_ERROR, format!("{kind}: null pointer"));
        return;
    }
    let size = measure(component, window);
    let buffer_size = native_window::geometry(window);
    let buffer_format = native_window::format(window);
    let Ok(mut state) = STATE.lock() else {
        log(hilog::LOG_ERROR, format!("{kind}: state lock poisoned"));
        return;
    };
    if state.registered_component != component as usize {
        log(hilog::LOG_WARN, format!("{kind}: stale component ignored"));
        return;
    }
    if kind == "created" || state.window != window as usize {
        state.generation += 1;
    }
    state.window = window as usize;
    let generation = state.generation;
    if let Ok((width, height)) = size {
        state.width = width;
        state.height = height;
    }
    // The previous on-screen Surface must be released before replacing the
    // window size or drawing on a newly created XComponent window.
    let old_drawing_released = state.drawing.take().is_some();
    let pixel_size = match buffer_size {
        Ok((width, height)) if width > 0 && height > 0 => Some((width, height)),
        _ => size.ok().and_then(|(width, height)| {
            Some((i32::try_from(width).ok()?, i32::try_from(height).ok()?))
        }),
    };
    if let Some((width, height)) = pixel_size {
        state.pixel_width = width;
        state.pixel_height = height;
    } else {
        state.pixel_width = 0;
        state.pixel_height = 0;
    }
    let draw_result = pixel_size.map(|(width, height)| {
        ScreenDrawing::create(window, width, height).map(|drawing| {
            state.drawing = Some(drawing);
            (width, height)
        })
    });
    drop(state);
    gpui::enqueue_surface(if kind == "created" {
        SurfaceEvent::Created(generation)
    } else {
        SurfaceEvent::Changed(generation)
    });
    if matches!(draw_result, Some(Ok(_))) {
        frame::surface_ready(component, generation);
    }
    match size {
        Ok((width, height)) => log(
            hilog::LOG_INFO,
            format!(
                "{kind}: generation={generation} component={component:p} window={window:p} size={width}x{height}vp thread={}",
                thread()
            ),
        ),
        Err(code) => log(
            hilog::LOG_ERROR,
            format!("{kind}: GetXComponentSize failed code={code} window={window:p}"),
        ),
    }
    log(
        hilog::LOG_INFO,
        format!(
            "{kind}: native_geometry={buffer_size:?} native_format={buffer_format:?} image_format=RGBA_8888({}) alpha=PREMUL({})",
            drawing::COLOR_RGBA_8888,
            drawing::ALPHA_PREMUL
        ),
    );
    match draw_result {
        Some(Ok((width, height))) => log(
            hilog::LOG_INFO,
            format!(
                "{kind}: Drawing Surface ready generation={generation} pixels={width}x{height} old_released={old_drawing_released}"
            ),
        ),
        Some(Err(error)) => log(
            hilog::LOG_ERROR,
            format!("{kind}: Drawing {} failed code={}", error.step, error.code),
        ),
        None => log(
            hilog::LOG_ERROR,
            format!("{kind}: no valid pixel dimensions; drawing skipped"),
        ),
    }
}

unsafe extern "C" fn on_created(component: *mut xc::XComponent, window: *mut c_void) {
    update_surface("created", component, window);
}

unsafe extern "C" fn on_changed(component: *mut xc::XComponent, window: *mut c_void) {
    update_surface("changed", component, window);
}

unsafe extern "C" fn on_destroyed(component: *mut xc::XComponent, window: *mut c_void) {
    let Ok(mut state) = STATE.lock() else {
        log(hilog::LOG_ERROR, "destroyed: state lock poisoned".into());
        return;
    };
    if state.registered_component != component as usize || state.window != window as usize {
        log(
            hilog::LOG_WARN,
            format!("destroyed: stale callback component={component:p} window={window:p}"),
        );
        return;
    }
    let generation = state.generation;
    let drawing_released = state.drawing.take().is_some();
    state.window = 0;
    state.width = 0;
    state.height = 0;
    state.pixel_width = 0;
    state.pixel_height = 0;
    drop(state);
    frame::surface_lost(component, generation);
    gpui::enqueue_surface(SurfaceEvent::Destroyed(generation));
    log(
        hilog::LOG_INFO,
        format!(
            "destroyed: generation={generation} window={window:p} drawing_released={drawing_released} thread={}",
            thread()
        ),
    );
}

unsafe extern "C" fn on_touch(component: *mut xc::XComponent, window: *mut c_void) {
    if component.is_null() || window.is_null() {
        log(hilog::LOG_ERROR, "touch: null pointer".into());
        return;
    }
    let Ok(state) = STATE.lock() else {
        log(hilog::LOG_ERROR, "touch: state lock poisoned".into());
        return;
    };
    if state.registered_component != component as usize || state.window != window as usize {
        log(hilog::LOG_WARN, "touch: stale Surface ignored".into());
        return;
    }
    let generation = state.generation;
    drop(state);
    touch::enqueue_event(component, window, generation);
}
