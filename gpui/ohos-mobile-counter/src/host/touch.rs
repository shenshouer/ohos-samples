//! Touch probe after the Surface callback validates the live window.

use std::ffi::c_void;

use crate::host::logging::log;
use crate::sdk_ffi::{hilog, xcomponent as xc};

pub(super) fn log_event(component: *mut xc::XComponent, window: *mut c_void, generation: u64) {
    // All-zero is valid for every field, including bool. The SDK fills the
    // structure on success, and failure never reads uninitialized memory.
    let mut event: xc::TouchEvent = unsafe { std::mem::zeroed() };
    // SAFETY: pointers are supplied by the SDK callback; event is writable.
    let result = unsafe { xc::OH_NativeXComponent_GetTouchEvent(component, window, &mut event) };
    if let Err(error) = xc::check_result(result) {
        log(
            hilog::LOG_ERROR,
            format!("touch: GetTouchEvent failed code={}", error.0),
        );
        return;
    }
    if event.num_points as usize > xc::MAX_TOUCH_POINTS {
        log(
            hilog::LOG_ERROR,
            format!("touch: invalid point count={}", event.num_points),
        );
        return;
    }
    log(
        hilog::LOG_INFO,
        format!(
            "touch: generation={generation} type={} points={} pos=({:.1},{:.1})vp thread={:?}",
            event.event_type,
            event.num_points,
            event.x,
            event.y,
            std::thread::current().id()
        ),
    );
}
