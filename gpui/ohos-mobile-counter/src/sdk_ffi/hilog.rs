//! `hilog/log.h` (API 18): fixed-argument logging from native callbacks.

use std::ffi::c_char;

pub const LOG_APP: i32 = 0;
pub const LOG_INFO: i32 = 4;
pub const LOG_WARN: i32 = 5;
pub const LOG_ERROR: i32 = 6;

#[link(name = "hilog_ndk.z")]
unsafe extern "C" {
    pub fn OH_LOG_PrintMsg(
        log_type: i32,
        level: i32,
        domain: u32,
        tag: *const c_char,
        message: *const c_char,
    ) -> i32;
}
