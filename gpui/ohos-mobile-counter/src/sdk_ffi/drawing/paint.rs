//! Canvas, brush and geometry interfaces from `native_drawing/*.h`.

#[repr(C)]
pub struct Canvas {
    _private: [u8; 0],
}

#[repr(C)]
pub struct Brush {
    _private: [u8; 0],
}

#[repr(C)]
pub struct Rect {
    _private: [u8; 0],
}

#[repr(C)]
pub struct RoundRect {
    _private: [u8; 0],
}

#[link(name = "native_drawing")]
unsafe extern "C" {
    pub fn OH_Drawing_CanvasClear(canvas: *mut Canvas, color: u32);
    pub fn OH_Drawing_CanvasAttachBrush(canvas: *mut Canvas, brush: *const Brush);
    pub fn OH_Drawing_CanvasDetachBrush(canvas: *mut Canvas);
    pub fn OH_Drawing_CanvasDrawRect(canvas: *mut Canvas, rect: *const Rect);
    pub fn OH_Drawing_CanvasDrawRoundRect(canvas: *mut Canvas, rect: *const RoundRect);
    pub fn OH_Drawing_BrushCreate() -> *mut Brush;
    pub fn OH_Drawing_BrushSetColor(brush: *mut Brush, color: u32);
    pub fn OH_Drawing_BrushSetAntiAlias(brush: *mut Brush, anti_alias: bool);
    pub fn OH_Drawing_BrushDestroy(brush: *mut Brush);
    pub fn OH_Drawing_RectCreate(left: f32, top: f32, right: f32, bottom: f32) -> *mut Rect;
    pub fn OH_Drawing_RectDestroy(rect: *mut Rect);
    pub fn OH_Drawing_RoundRectCreate(rect: *const Rect, x_rad: f32, y_rad: f32) -> *mut RoundRect;
    pub fn OH_Drawing_RoundRectDestroy(rect: *mut RoundRect);
}
