//! Demand-driven XComponent frame notifications for the embedded GPUI window.

use std::sync::Mutex;

use crate::gpui;
use crate::host::logging::log;
use crate::sdk_ffi::{hilog, xcomponent as xc};

struct FrameState {
    component: usize,
    generation: u64,
    registered: bool,
    active: bool,
    requests: u64,
    callbacks: u64,
}

static STATE: Mutex<FrameState> = Mutex::new(FrameState {
    component: 0,
    generation: 0,
    registered: false,
    active: true,
    requests: 0,
    callbacks: 0,
});

fn unregister(component: usize) -> bool {
    // SAFETY: the caller retained the SDK-owned component while its callback
    // registration was active. This runs outside STATE's lock.
    let result = unsafe {
        xc::OH_NativeXComponent_UnregisterOnFrameCallback(component as *mut xc::XComponent)
    };
    match xc::check_result(result) {
        Ok(()) => true,
        Err(error) => {
            log(
                hilog::LOG_ERROR,
                format!("XComponent frame unregister failed code={}", error.0),
            );
            false
        }
    }
}

pub(crate) fn surface_ready(component: *mut xc::XComponent, generation: u64) {
    let old = {
        let Ok(mut state) = STATE.lock() else { return };
        if state.component == component as usize && state.generation == generation {
            None
        } else {
            let old = state.registered.then_some(state.component);
            state.component = component as usize;
            state.generation = generation;
            state.registered = false;
            old
        }
    };
    if let Some(old) = old {
        let _ = unregister(old);
    }
    wake_for(generation);
}

pub(crate) fn surface_lost(component: *mut xc::XComponent, generation: u64) {
    let old = {
        let Ok(mut state) = STATE.lock() else { return };
        if state.component != component as usize || state.generation != generation {
            return;
        }
        let old = state.registered.then_some(state.component);
        state.component = 0;
        state.registered = false;
        old
    };
    let cancelled = old.map(unregister);
    log(
        hilog::LOG_INFO,
        format!(
            "XComponent frame surface lost generation={generation} pending_cancel_result={cancelled:?}"
        ),
    );
}

pub(crate) fn wake_for(generation: u64) {
    request(Some(generation));
}

pub(crate) fn wake() {
    request(None);
}

fn request(expected_generation: Option<u64>) {
    let (component, generation, ordinal) = {
        let Ok(mut state) = STATE.lock() else { return };
        if !state.active
            || state.component == 0
            || state.registered
            || expected_generation.is_some_and(|expected| expected != state.generation)
        {
            return;
        }
        state.registered = true;
        state.requests += 1;
        (state.component, state.generation, state.requests)
    };
    // SAFETY: component is SDK-owned and retained by the current Surface.
    let result = unsafe {
        xc::OH_NativeXComponent_RegisterOnFrameCallback(
            component as *mut xc::XComponent,
            Some(on_frame),
        )
    };
    if let Err(error) = xc::check_result(result) {
        if let Ok(mut state) = STATE.lock() {
            if state.component == component && state.generation == generation {
                state.registered = false;
            }
        }
        log(
            hilog::LOG_ERROR,
            format!(
                "XComponent frame request failed generation={generation} code={}",
                error.0
            ),
        );
    } else if ordinal <= 3 || ordinal % 120 == 0 {
        log(
            hilog::LOG_INFO,
            format!("XComponent frame requested generation={generation} ordinal={ordinal}"),
        );
    }
}

pub(crate) fn pause() {
    let (old, requests, callbacks) = {
        let Ok(mut state) = STATE.lock() else { return };
        state.active = false;
        let old = state.registered.then_some(state.component);
        state.registered = false;
        (old, state.requests, state.callbacks)
    };
    let cancelled = old.map(unregister);
    log(
        hilog::LOG_INFO,
        format!(
            "GPUI frame paused requests={requests} callbacks={callbacks} pending_cancel_result={cancelled:?}"
        ),
    );
}

pub(crate) fn resume() {
    if let Ok(mut state) = STATE.lock() {
        state.active = true;
    }
    log(hilog::LOG_INFO, "GPUI frame resumed".into());
    wake();
}

pub(crate) fn is_active() -> bool {
    STATE.lock().ok().is_some_and(|state| state.active)
}

unsafe extern "C" fn on_frame(
    component: *mut xc::XComponent,
    timestamp: u64,
    target_timestamp: u64,
) {
    let (generation, ordinal) = {
        let Ok(mut state) = STATE.lock() else { return };
        if !state.active || !state.registered || state.component != component as usize {
            return;
        }
        state.registered = false;
        state.callbacks += 1;
        (state.generation, state.callbacks)
    };
    let _ = unregister(component as usize);
    if ordinal <= 3 || ordinal % 120 == 0 {
        log(
            hilog::LOG_INFO,
            format!(
                "XComponent frame callback generation={generation} ordinal={ordinal} timestamp={timestamp} target={target_timestamp} thread={:?}",
                std::thread::current().id()
            ),
        );
    }
    gpui::pump_frame();
}
