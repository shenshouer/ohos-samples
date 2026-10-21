//! Minimal `native_window/external_window.h` queries for M2-1.

use core::ffi::c_void;

// `NativeWindowOperation` values in the API 26 SDK.
pub const GET_BUFFER_GEOMETRY: i32 = 1;
pub const GET_FORMAT: i32 = 2;

#[link(name = "native_window")]
unsafe extern "C" {
    pub fn OH_NativeWindow_NativeWindowHandleOpt(window: *mut c_void, code: i32, ...) -> i32;
}

pub fn geometry(window: *mut c_void) -> Result<(i32, i32), i32> {
    let (mut height, mut width) = (0, 0);
    // SAFETY: `window` is the SDK-provided NativeWindow during its callback;
    // GET_BUFFER_GEOMETRY expects height and then width output pointers.
    let result = unsafe {
        OH_NativeWindow_NativeWindowHandleOpt(
            window,
            GET_BUFFER_GEOMETRY,
            &mut height as *mut i32,
            &mut width as *mut i32,
        )
    };
    if result == 0 {
        Ok((width, height))
    } else {
        Err(result)
    }
}

pub fn format(window: *mut c_void) -> Result<i32, i32> {
    let mut format = 0;
    // SAFETY: GET_FORMAT expects one writable i32 output pointer.
    let result = unsafe {
        OH_NativeWindow_NativeWindowHandleOpt(window, GET_FORMAT, &mut format as *mut i32)
    };
    if result == 0 { Ok(format) } else { Err(result) }
}
