//! M2-1: one Native Drawing frame on an XComponent NativeWindow.

use std::ffi::c_void;

use crate::sdk_ffi::drawing as d;

const BACKGROUND: u32 = 0xFF10243A;
const CARD: u32 = 0xFF35C7ED;

#[derive(Debug)]
pub struct ProbeError {
    pub step: &'static str,
    pub code: i32,
}

fn status(step: &'static str) -> Result<(), ProbeError> {
    // SAFETY: Drawing maintains the error status for the calling thread.
    let code = unsafe { d::OH_Drawing_ErrorCodeGet() };
    d::check_result(code).map_err(|error| ProbeError {
        step,
        code: error.0,
    })
}

fn null_pointer(step: &'static str) -> ProbeError {
    // Some allocation failures return null without setting a Drawing code.
    let code = unsafe { d::OH_Drawing_ErrorCodeGet() };
    ProbeError {
        step,
        code: if code == d::SUCCESS { -1 } else { code },
    }
}

fn call_void(step: &'static str, call: impl FnOnce()) -> Result<(), ProbeError> {
    // SAFETY: Reset before each void API so a previous failure is not
    // incorrectly attributed to the next operation.
    unsafe { d::OH_Drawing_ErrorCodeReset() };
    call();
    status(step)
}

pub struct ProbeDrawing {
    context: usize,
    surface: usize,
}

impl Drop for ProbeDrawing {
    fn drop(&mut self) {
        // SAFETY: these handles were created by Native Drawing and are owned
        // by this wrapper. The Surface must be released before its context.
        unsafe {
            if self.surface != 0 {
                d::OH_Drawing_SurfaceDestroy(self.surface as *mut d::Surface);
            }
            if self.context != 0 {
                d::OH_Drawing_GpuContextDestroy(self.context as *mut d::GpuContext);
            }
        }
    }
}

impl ProbeDrawing {
    pub fn create(window: *mut c_void, width: i32, height: i32) -> Result<Self, ProbeError> {
        if window.is_null() || width <= 0 || height <= 0 {
            return Err(ProbeError {
                step: "image_info",
                code: 401,
            });
        }
        // SAFETY: no pointer parameters; return value is checked below.
        unsafe { d::OH_Drawing_ErrorCodeReset() };
        let context = unsafe { d::OH_Drawing_GpuContextCreate() };
        let mut owned = Self {
            context: context as usize,
            surface: 0,
        };
        if context.is_null() {
            return Err(null_pointer("gpu_context_create"));
        }
        status("gpu_context_create")?;

        let info = d::ImageInfo {
            width,
            height,
            color_type: d::COLOR_RGBA_8888,
            alpha_type: d::ALPHA_PREMUL,
        };
        // SAFETY: the XComponent window remains valid throughout its callback;
        // owned keeps the GPU context alive for the Surface lifetime.
        unsafe { d::OH_Drawing_ErrorCodeReset() };
        let surface = unsafe { d::OH_Drawing_SurfaceCreateOnScreen(context, info, window) };
        owned.surface = surface as usize;
        if surface.is_null() {
            return Err(null_pointer("surface_create_on_screen"));
        }
        status("surface_create_on_screen")?;
        owned.paint(width, height)?;
        Ok(owned)
    }

    fn paint(&mut self, width: i32, height: i32) -> Result<(), ProbeError> {
        // SAFETY: Surface is owned by self and remains live during this call.
        unsafe { d::OH_Drawing_ErrorCodeReset() };
        let canvas = unsafe { d::OH_Drawing_SurfaceGetCanvas(self.surface as *mut d::Surface) };
        if canvas.is_null() {
            return Err(null_pointer("surface_get_canvas"));
        }
        status("surface_get_canvas")?;
        call_void("canvas_clear", || unsafe {
            d::OH_Drawing_CanvasClear(canvas, BACKGROUND)
        })?;

        let mut shape = ShapeObjects::default();
        unsafe { d::OH_Drawing_ErrorCodeReset() };
        let brush = unsafe { d::OH_Drawing_BrushCreate() };
        shape.brush = brush as usize;
        if brush.is_null() {
            return Err(null_pointer("brush_create"));
        }
        status("brush_create")?;
        call_void("brush_set_color", || unsafe {
            d::OH_Drawing_BrushSetColor(brush, CARD)
        })?;
        call_void("brush_set_antialias", || unsafe {
            d::OH_Drawing_BrushSetAntiAlias(brush, true)
        })?;

        let width = width as f32;
        let height = height as f32;
        let top = height * 0.35;
        let bottom = height * 0.65;
        unsafe { d::OH_Drawing_ErrorCodeReset() };
        let rect = unsafe { d::OH_Drawing_RectCreate(width * 0.12, top, width * 0.88, bottom) };
        shape.rect = rect as usize;
        if rect.is_null() {
            return Err(null_pointer("rect_create"));
        }
        status("rect_create")?;
        unsafe { d::OH_Drawing_ErrorCodeReset() };
        let round_rect = unsafe { d::OH_Drawing_RoundRectCreate(rect, 48.0, 48.0) };
        shape.round_rect = round_rect as usize;
        if round_rect.is_null() {
            return Err(null_pointer("round_rect_create"));
        }
        status("round_rect_create")?;

        call_void("canvas_attach_brush", || unsafe {
            d::OH_Drawing_CanvasAttachBrush(canvas, brush)
        })?;
        let draw_result = call_void("canvas_draw_round_rect", || unsafe {
            d::OH_Drawing_CanvasDrawRoundRect(canvas, round_rect)
        });
        let detach_result = call_void("canvas_detach_brush", || unsafe {
            d::OH_Drawing_CanvasDetachBrush(canvas)
        });
        draw_result?;
        detach_result?;
        drop(shape);

        // SAFETY: this Surface came from SurfaceCreateOnScreen.
        unsafe { d::OH_Drawing_ErrorCodeReset() };
        let flush_code = unsafe { d::OH_Drawing_SurfaceFlush(self.surface as *mut d::Surface) };
        d::check_result(flush_code).map_err(|error| ProbeError {
            step: "surface_flush",
            code: error.0,
        })?;
        status("surface_flush")
    }
}

#[derive(Default)]
struct ShapeObjects {
    brush: usize,
    rect: usize,
    round_rect: usize,
}

impl Drop for ShapeObjects {
    fn drop(&mut self) {
        // SAFETY: each nonzero pointer is owned by this temporary wrapper.
        unsafe {
            if self.round_rect != 0 {
                d::OH_Drawing_RoundRectDestroy(self.round_rect as *mut d::RoundRect);
            }
            if self.rect != 0 {
                d::OH_Drawing_RectDestroy(self.rect as *mut d::Rect);
            }
            if self.brush != 0 {
                d::OH_Drawing_BrushDestroy(self.brush as *mut d::Brush);
            }
        }
    }
}
