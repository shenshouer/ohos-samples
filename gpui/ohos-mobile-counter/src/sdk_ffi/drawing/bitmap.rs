//! Bitmap APIs used to paint the project's monochrome glyph atlas.

use super::{Canvas, Rect};

#[repr(C)]
pub struct Bitmap {
    _private: [u8; 0],
}

#[repr(C)]
pub struct SamplingOptions {
    _private: [u8; 0],
}

#[repr(C)]
pub struct BitmapFormat {
    pub color_format: i32,
    pub alpha_format: i32,
}

#[link(name = "native_drawing")]
unsafe extern "C" {
    pub fn OH_Drawing_BitmapCreate() -> *mut Bitmap;
    pub fn OH_Drawing_BitmapDestroy(bitmap: *mut Bitmap);
    pub fn OH_Drawing_BitmapBuild(
        bitmap: *mut Bitmap,
        width: u32,
        height: u32,
        format: *const BitmapFormat,
    );
    pub fn OH_Drawing_BitmapGetPixels(bitmap: *mut Bitmap) -> *mut u8;
    pub fn OH_Drawing_BitmapGetRowBytes(bitmap: *mut Bitmap, bytes: *mut u32) -> i32;
    pub fn OH_Drawing_CanvasDrawBitmapRect(
        canvas: *mut Canvas,
        bitmap: *const Bitmap,
        src: *const Rect,
        dst: *const Rect,
        sampling: *const SamplingOptions,
    );
}

const _: () = {
    use core::mem::{align_of, size_of};
    assert!(size_of::<BitmapFormat>() == 8);
    assert!(align_of::<BitmapFormat>() == 4);
};
