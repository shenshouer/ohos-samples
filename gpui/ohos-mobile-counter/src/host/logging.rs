//! Native callback logging through the SDK hiLog interface.

use std::ffi::CString;

use crate::sdk_ffi::hilog;

const TAG: &[u8] = b"GPUI_M4_1\0";

pub(crate) fn log(level: i32, message: String) {
    let Ok(message) = CString::new(message) else {
        return;
    };
    // SAFETY: pointers refer to NUL-terminated strings alive for this call.
    let result = unsafe {
        hilog::OH_LOG_PrintMsg(
            hilog::LOG_APP,
            level,
            0,
            TAG.as_ptr().cast(),
            message.as_ptr(),
        )
    };
    if result < 0 {
        eprintln!("GPUI_M4_1: OH_LOG_PrintMsg failed: {result}");
    }
}
