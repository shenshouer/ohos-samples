//! `native_drawing/drawing_error_code.h`.

pub const SUCCESS: i32 = 0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DrawingError(pub i32);

pub fn check_result(code: i32) -> Result<(), DrawingError> {
    if code == SUCCESS {
        Ok(())
    } else {
        Err(DrawingError(code))
    }
}

#[link(name = "native_drawing")]
unsafe extern "C" {
    // Get introduced in API 12, Reset in API 18.
    pub fn OH_Drawing_ErrorCodeGet() -> i32;
    pub fn OH_Drawing_ErrorCodeReset();
}
