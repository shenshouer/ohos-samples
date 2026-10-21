//! Ordered GPUI scene batches rendered with the system Native Drawing canvas.

use std::ptr;

use crate::gpui::atlas::MonoAtlas;
use crate::gpui_core::{
    Bounds, Hsla, MonochromeSprite, PrimitiveBatch, Quad, Result, Rgba, ScaledPixels, Scene,
    TransformationMatrix,
};
use crate::sdk_ffi::drawing as d;

pub(crate) struct PaintStats {
    pub(crate) quads: usize,
    pub(crate) glyphs: usize,
}

fn native(step: &'static str) -> Result<()> {
    let code = unsafe { d::OH_Drawing_ErrorCodeGet() };
    d::check_result(code).map_err(|error| {
        crate::gpui_core::private::anyhow::anyhow!("Native Drawing {step} failed: {}", error.0)
    })
}

fn call(step: &'static str, f: impl FnOnce()) -> Result<()> {
    unsafe { d::OH_Drawing_ErrorCodeReset() };
    f();
    native(step)
}

struct Owned<T>(*mut T, unsafe extern "C" fn(*mut T));

impl<T> Owned<T> {
    fn new(ptr: *mut T, destroy: unsafe extern "C" fn(*mut T), step: &'static str) -> Result<Self> {
        if ptr.is_null() {
            return Err(crate::gpui_core::private::anyhow::anyhow!(
                "Native Drawing {step} returned null"
            ));
        }
        let owned = Self(ptr, destroy);
        native(step)?;
        Ok(owned)
    }

    fn ptr(&self) -> *mut T {
        self.0
    }
}

impl<T> Drop for Owned<T> {
    fn drop(&mut self) {
        unsafe { (self.1)(self.0) };
    }
}

fn rect(left: f32, top: f32, right: f32, bottom: f32) -> Result<Owned<d::Rect>> {
    if ![left, top, right, bottom].into_iter().all(f32::is_finite) || right <= left || bottom <= top
    {
        return Err(crate::gpui_core::private::anyhow::anyhow!(
            "invalid Drawing rectangle"
        ));
    }
    unsafe { d::OH_Drawing_ErrorCodeReset() };
    Owned::new(
        unsafe { d::OH_Drawing_RectCreate(left, top, right, bottom) },
        d::OH_Drawing_RectDestroy,
        "rect_create",
    )
}

fn bounds_rect(bounds: Bounds<ScaledPixels>) -> Result<Owned<d::Rect>> {
    let left = bounds.origin.x.0;
    let top = bounds.origin.y.0;
    rect(
        left,
        top,
        left + bounds.size.width.0,
        top + bounds.size.height.0,
    )
}

fn argb(color: Hsla) -> u32 {
    let rgba = Rgba::from(color);
    let channel = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as u32;
    channel(rgba.a) << 24 | channel(rgba.r) << 16 | channel(rgba.g) << 8 | channel(rgba.b)
}

fn with_clip<R>(
    canvas: *mut d::Canvas,
    bounds: Bounds<ScaledPixels>,
    paint: impl FnOnce() -> Result<R>,
) -> Result<R> {
    let clip = bounds_rect(bounds)?;
    call("canvas_save", || unsafe {
        d::OH_Drawing_CanvasSave(canvas)
    })?;
    let result = call("canvas_clip_rect", || unsafe {
        d::OH_Drawing_CanvasClipRect(canvas, clip.ptr(), d::CLIP_INTERSECT, false)
    })
    .and_then(|()| paint());
    let restored = call("canvas_restore", || unsafe {
        d::OH_Drawing_CanvasRestore(canvas)
    });
    match (result, restored) {
        (Err(error), _) => Err(error),
        (Ok(_), Err(error)) => Err(error),
        (Ok(value), Ok(())) => Ok(value),
    }
}

fn draw_shape(canvas: *mut d::Canvas, bounds: Bounds<ScaledPixels>, radius: f32) -> Result<()> {
    let rectangle = bounds_rect(bounds)?;
    if radius > 0.0 {
        unsafe { d::OH_Drawing_ErrorCodeReset() };
        let rounded = Owned::new(
            unsafe { d::OH_Drawing_RoundRectCreate(rectangle.ptr(), radius, radius) },
            d::OH_Drawing_RoundRectDestroy,
            "round_rect_create",
        )?;
        call("canvas_draw_round_rect", || unsafe {
            d::OH_Drawing_CanvasDrawRoundRect(canvas, rounded.ptr())
        })
    } else {
        call("canvas_draw_rect", || unsafe {
            d::OH_Drawing_CanvasDrawRect(canvas, rectangle.ptr())
        })
    }
}

fn uniform(values: [f32; 4], kind: &'static str) -> Result<f32> {
    if values
        .iter()
        .any(|value| !value.is_finite() || *value < 0.0)
        || values.iter().any(|value| (value - values[0]).abs() > 0.01)
    {
        return Err(crate::gpui_core::private::anyhow::anyhow!(
            "nonuniform or invalid {kind} unsupported in M3-2"
        ));
    }
    Ok(values[0])
}

fn draw_quad(canvas: *mut d::Canvas, quad: &Quad) -> Result<()> {
    let radii = quad.corner_radii;
    let radius = uniform(
        [
            radii.top_left.0,
            radii.top_right.0,
            radii.bottom_right.0,
            radii.bottom_left.0,
        ],
        "corner radii",
    )?;
    let widths = quad.border_widths;
    let border = uniform(
        [widths.top.0, widths.right.0, widths.bottom.0, widths.left.0],
        "border widths",
    )?;
    if quad.border_style != crate::gpui_core::BorderStyle::Solid && border > 0.0 {
        return Err(crate::gpui_core::private::anyhow::anyhow!(
            "dashed border unsupported in M3-2"
        ));
    }
    let background = quad.background.as_solid().ok_or_else(|| {
        crate::gpui_core::private::anyhow::anyhow!("non-solid background unsupported in M3-2")
    })?;
    with_clip(canvas, quad.content_mask.bounds, || {
        if background.a > 0.0 {
            unsafe { d::OH_Drawing_ErrorCodeReset() };
            let brush = Owned::new(
                unsafe { d::OH_Drawing_BrushCreate() },
                d::OH_Drawing_BrushDestroy,
                "brush_create",
            )?;
            call("brush_color", || unsafe {
                d::OH_Drawing_BrushSetColor(brush.ptr(), argb(background))
            })?;
            call("brush_antialias", || unsafe {
                d::OH_Drawing_BrushSetAntiAlias(brush.ptr(), true)
            })?;
            call("attach_brush", || unsafe {
                d::OH_Drawing_CanvasAttachBrush(canvas, brush.ptr())
            })?;
            let painted = draw_shape(canvas, quad.bounds, radius);
            let detached = call("detach_brush", || unsafe {
                d::OH_Drawing_CanvasDetachBrush(canvas)
            });
            painted?;
            detached?;
        }
        if border > 0.0 && quad.border_color.a > 0.0 {
            unsafe { d::OH_Drawing_ErrorCodeReset() };
            let pen = Owned::new(
                unsafe { d::OH_Drawing_PenCreate() },
                d::OH_Drawing_PenDestroy,
                "pen_create",
            )?;
            call("pen_color", || unsafe {
                d::OH_Drawing_PenSetColor(pen.ptr(), argb(quad.border_color))
            })?;
            call("pen_width", || unsafe {
                d::OH_Drawing_PenSetWidth(pen.ptr(), border)
            })?;
            call("pen_antialias", || unsafe {
                d::OH_Drawing_PenSetAntiAlias(pen.ptr(), true)
            })?;
            call("attach_pen", || unsafe {
                d::OH_Drawing_CanvasAttachPen(canvas, pen.ptr())
            })?;
            let inset = border * 0.5;
            let mut inner = quad.bounds;
            inner.origin.x.0 += inset;
            inner.origin.y.0 += inset;
            inner.size.width.0 -= border;
            inner.size.height.0 -= border;
            let painted = draw_shape(canvas, inner, (radius - inset).max(0.0));
            let detached = call("detach_pen", || unsafe {
                d::OH_Drawing_CanvasDetachPen(canvas)
            });
            painted?;
            detached?;
        }
        Ok(())
    })
}

fn draw_sprite(canvas: *mut d::Canvas, sprite: &MonochromeSprite, atlas: &MonoAtlas) -> Result<()> {
    if sprite.transformation != TransformationMatrix::unit() {
        return Err(crate::gpui_core::private::anyhow::anyhow!(
            "transformed monochrome sprite unsupported in M3-2"
        ));
    }
    let mask = atlas.mask(sprite.tile)?;
    let row_pixels = usize::try_from(mask.width)?;
    let rows = usize::try_from(mask.height)?;
    unsafe { d::OH_Drawing_ErrorCodeReset() };
    let bitmap = Owned::new(
        unsafe { d::OH_Drawing_BitmapCreate() },
        d::OH_Drawing_BitmapDestroy,
        "bitmap_create",
    )?;
    let format = d::BitmapFormat {
        color_format: d::COLOR_RGBA_8888,
        alpha_format: d::ALPHA_PREMUL,
    };
    call("bitmap_build", || unsafe {
        d::OH_Drawing_BitmapBuild(bitmap.ptr(), mask.width, mask.height, &format)
    })?;
    let mut row_bytes = 0;
    let stride_code = unsafe { d::OH_Drawing_BitmapGetRowBytes(bitmap.ptr(), &mut row_bytes) };
    d::check_result(stride_code).map_err(|error| {
        crate::gpui_core::private::anyhow::anyhow!(
            "Native Drawing bitmap row bytes failed: {}",
            error.0
        )
    })?;
    let stride = usize::try_from(row_bytes)?;
    if stride < row_pixels * 4 {
        return Err(crate::gpui_core::private::anyhow::anyhow!(
            "glyph bitmap stride shorter than RGBA row"
        ));
    }
    let byte_count = stride
        .checked_mul(rows)
        .ok_or_else(|| crate::gpui_core::private::anyhow::anyhow!("glyph bitmap size overflow"))?;
    unsafe { d::OH_Drawing_ErrorCodeReset() };
    let pixels = unsafe { d::OH_Drawing_BitmapGetPixels(bitmap.ptr()) };
    if pixels.is_null() {
        return Err(crate::gpui_core::private::anyhow::anyhow!(
            "bitmap pixels unavailable"
        ));
    }
    native("bitmap_get_pixels")?;
    let rgba = Rgba::from(sprite.color);
    let channel = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as u8;
    let alpha = channel(rgba.a);
    let red = channel(rgba.r);
    let green = channel(rgba.g);
    let blue = channel(rgba.b);
    // SAFETY: BitmapBuild allocated row_bytes * height bytes. The SDK-reported
    // stride and checked dimensions keep every RGBA pixel within that buffer.
    let target = unsafe { std::slice::from_raw_parts_mut(pixels, byte_count) };
    for y in 0..rows {
        for x in 0..row_pixels {
            let src = mask.alpha[y * row_pixels + x];
            let dst = &mut target[y * stride + x * 4..][..4];
            let a = (u16::from(src) * u16::from(alpha) / 255) as u8;
            dst.copy_from_slice(&[
                (u16::from(red) * u16::from(a) / 255) as u8,
                (u16::from(green) * u16::from(a) / 255) as u8,
                (u16::from(blue) * u16::from(a) / 255) as u8,
                a,
            ]);
        }
    }
    with_clip(canvas, sprite.content_mask.bounds, || {
        let dst = bounds_rect(sprite.bounds)?;
        call("draw_glyph_bitmap", || unsafe {
            d::OH_Drawing_CanvasDrawBitmapRect(
                canvas,
                bitmap.ptr(),
                ptr::null(),
                dst.ptr(),
                ptr::null(),
            )
        })
    })
}

pub(crate) fn paint(
    canvas: *mut d::Canvas,
    scene: &Scene,
    atlas: &MonoAtlas,
) -> Result<PaintStats> {
    if canvas.is_null() {
        return Err(crate::gpui_core::private::anyhow::anyhow!(
            "null Drawing canvas"
        ));
    }
    let mut stats = PaintStats {
        quads: 0,
        glyphs: 0,
    };
    call("canvas_clear", || unsafe {
        d::OH_Drawing_CanvasClear(canvas, 0xff10243a)
    })?;
    for batch in scene.batches() {
        match batch {
            PrimitiveBatch::Quads(range) => {
                for quad in &scene.quads[range] {
                    draw_quad(canvas, quad)?;
                    stats.quads += 1;
                }
            }
            PrimitiveBatch::MonochromeSprites { range, .. } => {
                for sprite in &scene.monochrome_sprites[range] {
                    draw_sprite(canvas, sprite, atlas)?;
                    stats.glyphs += 1;
                }
            }
            other => {
                return Err(crate::gpui_core::private::anyhow::anyhow!(
                    "unsupported GPUI primitive batch: {}",
                    other.label()
                ));
            }
        }
    }
    Ok(stats)
}
