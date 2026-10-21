//! Font and text-blob C ABI copied from the installed Native Drawing headers.

use core::ffi::c_void;

use super::{Canvas, Rect};

#[repr(C)]
pub struct Font {
    _private: [u8; 0],
}

#[repr(C)]
pub struct Typeface {
    _private: [u8; 0],
}

#[repr(C)]
pub struct TextBlob {
    _private: [u8; 0],
}

#[repr(C)]
#[derive(Default)]
pub struct FontMetrics {
    pub flags: u32,
    pub top: f32,
    pub ascent: f32,
    pub descent: f32,
    pub bottom: f32,
    pub leading: f32,
    pub avg_char_width: f32,
    pub max_char_width: f32,
    pub x_min: f32,
    pub x_max: f32,
    pub x_height: f32,
    pub cap_height: f32,
    pub underline_thickness: f32,
    pub underline_position: f32,
    pub strikeout_thickness: f32,
    pub strikeout_position: f32,
}

pub const TEXT_UTF8: i32 = 0;
pub const TEXT_GLYPH_ID: i32 = 3;
pub const COLOR_ALPHA_8: i32 = 1;

#[link(name = "native_drawing")]
unsafe extern "C" {
    pub fn OH_Drawing_TypefaceCreateDefault() -> *mut Typeface;
    pub fn OH_Drawing_TypefaceDestroy(typeface: *mut Typeface);
    pub fn OH_Drawing_FontCreate() -> *mut Font;
    pub fn OH_Drawing_FontDestroy(font: *mut Font);
    pub fn OH_Drawing_FontSetTypeface(font: *mut Font, typeface: *mut Typeface);
    pub fn OH_Drawing_FontSetTextSize(font: *mut Font, size: f32);
    pub fn OH_Drawing_FontSetEdging(font: *mut Font, edging: i32);
    pub fn OH_Drawing_FontTextToGlyphs(
        font: *const Font,
        text: *const c_void,
        byte_length: u32,
        encoding: i32,
        glyphs: *mut u16,
        max_glyph_count: i32,
    ) -> u32;
    pub fn OH_Drawing_FontGetWidths(
        font: *const Font,
        glyphs: *const u16,
        count: i32,
        widths: *mut f32,
    );
    pub fn OH_Drawing_FontGetMetrics(font: *mut Font, metrics: *mut FontMetrics) -> f32;
    pub fn OH_Drawing_TextBlobCreateFromText(
        text: *const c_void,
        byte_length: usize,
        font: *const Font,
        encoding: i32,
    ) -> *mut TextBlob;
    pub fn OH_Drawing_TextBlobGetBounds(blob: *mut TextBlob, rect: *mut Rect);
    pub fn OH_Drawing_TextBlobDestroy(blob: *mut TextBlob);
    pub fn OH_Drawing_RectGetLeft(rect: *mut Rect) -> f32;
    pub fn OH_Drawing_RectGetTop(rect: *mut Rect) -> f32;
    pub fn OH_Drawing_RectGetRight(rect: *mut Rect) -> f32;
    pub fn OH_Drawing_RectGetBottom(rect: *mut Rect) -> f32;
    pub fn OH_Drawing_CanvasCreate() -> *mut Canvas;
    pub fn OH_Drawing_CanvasDestroy(canvas: *mut Canvas);
    pub fn OH_Drawing_CanvasBind(canvas: *mut Canvas, bitmap: *mut super::Bitmap);
    pub fn OH_Drawing_CanvasDrawTextBlob(
        canvas: *mut Canvas,
        blob: *const TextBlob,
        x: f32,
        y: f32,
    );
}

const _: () = {
    use core::mem::{align_of, size_of};
    assert!(size_of::<FontMetrics>() == 64);
    assert!(align_of::<FontMetrics>() == 4);
};
