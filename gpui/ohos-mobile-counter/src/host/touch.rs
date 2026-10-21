//! Copy SDK touch data in the callback; GPUI receives it later on the UI pump.

use std::ffi::c_void;

use crate::gpui::{self, TouchInput};
use crate::gpui_core::TouchPhase;
use crate::host::frame;
use crate::host::logging::log;
use crate::sdk_ffi::{hilog, xcomponent as xc};

pub(super) fn enqueue_event(component: *mut xc::XComponent, window: *mut c_void, generation: u64) {
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
    let phase = match event.event_type {
        xc::TOUCH_DOWN => TouchPhase::Started,
        xc::TOUCH_MOVE => TouchPhase::Moved,
        xc::TOUCH_UP => TouchPhase::Ended,
        xc::TOUCH_CANCEL => TouchPhase::Cancelled,
        other => {
            log(
                hilog::LOG_WARN,
                format!("touch: unknown event type={other}"),
            );
            return;
        }
    };
    if !event.x.is_finite() || !event.y.is_finite() {
        log(hilog::LOG_ERROR, "touch: non-finite coordinate".into());
        return;
    }
    if phase != TouchPhase::Moved {
        log(
            hilog::LOG_INFO,
            format!(
                "touch: generation={generation} id={} phase={phase:?} points={} pos=({:.1},{:.1})px thread={:?}",
                event.id,
                event.num_points,
                event.x,
                event.y,
                std::thread::current().id()
            ),
        );
    }
    gpui::enqueue_touch(TouchInput {
        window: window as usize,
        generation,
        source_id: event.id,
        phase,
        x_pixels: event.x,
        y_pixels: event.y,
        force: event
            .force
            .is_finite()
            .then_some(event.force.clamp(0.0, 1.0)),
    });
    frame::wake_for(generation);
}
