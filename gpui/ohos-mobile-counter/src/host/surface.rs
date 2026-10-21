//! XComponent lifecycle state and synchronous SDK callback adapters.

use std::ffi::c_void;
use std::sync::Mutex;

use crate::host::logging::log;
use crate::host::touch;
use crate::render::probe::ProbeDrawing;
use crate::sdk_ffi::{drawing, hilog, native_window, xcomponent as xc};

#[derive(Default)]
struct SurfaceState {
    registered_component: usize,
    window: usize,
    generation: u64,
    width: u64,
    height: u64,
    drawing: Option<ProbeDrawing>,
}

static STATE: Mutex<SurfaceState> = Mutex::new(SurfaceState {
    registered_component: 0,
    window: 0,
    generation: 0,
    width: 0,
    height: 0,
    drawing: None,
});

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
    let draw_result = pixel_size.map(|(width, height)| {
        ProbeDrawing::create(window, width, height).map(|drawing| {
            state.drawing = Some(drawing);
            (width, height)
        })
    });
    drop(state);
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
                "{kind}: Drawing flush ok generation={generation} pixels={width}x{height} old_released={old_drawing_released}"
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
    drop(state);
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
    touch::log_event(component, window, generation);
}
