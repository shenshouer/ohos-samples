//! Touch structures and event query on a live XComponent Surface.

use core::ffi::c_void;

use super::surface::XComponent;

pub const MAX_TOUCH_POINTS: usize = 10;
pub const TOUCH_DOWN: i32 = 0;
pub const TOUCH_UP: i32 = 1;
pub const TOUCH_MOVE: i32 = 2;
pub const TOUCH_CANCEL: i32 = 3;

/// A C enum is held as `i32` so unknown SDK values do not create an invalid
/// Rust enum discriminant.
pub type TouchEventType = i32;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TouchPoint {
    pub id: i32,
    pub screen_x: f32,
    pub screen_y: f32,
    pub x: f32,
    pub y: f32,
    pub event_type: TouchEventType,
    pub size: f64,
    pub force: f32,
    pub timestamp: i64,
    pub is_pressed: bool,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TouchEvent {
    pub id: i32,
    pub screen_x: f32,
    pub screen_y: f32,
    pub x: f32,
    pub y: f32,
    pub event_type: TouchEventType,
    pub size: f64,
    pub force: f32,
    pub device_id: i64,
    pub timestamp: i64,
    pub touch_points: [TouchPoint; MAX_TOUCH_POINTS],
    pub num_points: u32,
}

#[link(name = "ace_ndk.z")]
unsafe extern "C" {
    pub fn OH_NativeXComponent_GetTouchEvent(
        component: *mut XComponent,
        window: *const c_void,
        event: *mut TouchEvent,
    ) -> i32;
}

const _: () = {
    use core::mem::{offset_of, size_of};
    assert!(size_of::<TouchPoint>() == 56);
    assert!(offset_of!(TouchPoint, size) == 24);
    assert!(offset_of!(TouchPoint, timestamp) == 40);
    assert!(size_of::<TouchEvent>() == 624);
    assert!(offset_of!(TouchEvent, touch_points) == 56);
    assert!(offset_of!(TouchEvent, num_points) == 616);
};
