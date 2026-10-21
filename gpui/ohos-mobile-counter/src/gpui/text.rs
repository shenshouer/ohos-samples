//! Minimal system-font text for the counter's Latin and minus glyphs.

use std::borrow::Cow;
use std::ffi::c_void;

use crate::gpui_core::*;
use crate::sdk_ffi::drawing as d;

pub(crate) struct OhosTextSystem;

struct Owned<T>(*mut T, unsafe extern "C" fn(*mut T));
impl<T> Owned<T> {
    fn new(ptr: *mut T, destroy: unsafe extern "C" fn(*mut T)) -> Result<Self> {
        if ptr.is_null() {
            return Err(crate::gpui_core::private::anyhow::anyhow!(
                "Native Drawing returned null"
            ));
        }
        let owned = Self(ptr, destroy);
        check()?;
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

fn reset() {
    unsafe { d::OH_Drawing_ErrorCodeReset() };
}
fn check() -> Result<()> {
    let code = unsafe { d::OH_Drawing_ErrorCodeGet() };
    d::check_result(code).map_err(|error| {
        crate::gpui_core::private::anyhow::anyhow!("Native Drawing text error {}", error.0)
    })
}

struct NativeFont {
    // Font holds a reference to the typeface; drop it first.
    font: Owned<d::Font>,
    _face: Owned<d::Typeface>,
}
impl NativeFont {
    fn new(size: f32) -> Result<Self> {
        if !size.is_finite() || size <= 0.0 {
            return Err(crate::gpui_core::private::anyhow::anyhow!(
                "invalid text size"
            ));
        }
        reset();
        let face = Owned::new(
            unsafe { d::OH_Drawing_TypefaceCreateDefault() },
            d::OH_Drawing_TypefaceDestroy,
        )?;
        reset();
        let font = Owned::new(
            unsafe { d::OH_Drawing_FontCreate() },
            d::OH_Drawing_FontDestroy,
        )?;
        reset();
        unsafe {
            d::OH_Drawing_FontSetTypeface(font.ptr(), face.ptr());
            d::OH_Drawing_FontSetTextSize(font.ptr(), size);
            d::OH_Drawing_FontSetEdging(font.ptr(), 1);
        }
        check()?;
        Ok(Self { font, _face: face })
    }
    fn glyph(&self, ch: char) -> Result<Option<u16>> {
        let mut utf8 = [0; 4];
        let text = ch.encode_utf8(&mut utf8);
        let mut glyph = 0u16;
        reset();
        let count = unsafe {
            d::OH_Drawing_FontTextToGlyphs(
                self.font.ptr(),
                text.as_ptr().cast::<c_void>(),
                text.len() as u32,
                d::TEXT_UTF8,
                &mut glyph,
                1,
            )
        };
        check()?;
        Ok((count == 1 && glyph != 0).then_some(glyph))
    }
    fn width(&self, glyph: u16) -> Result<f32> {
        let mut width = 0.0;
        reset();
        unsafe { d::OH_Drawing_FontGetWidths(self.font.ptr(), &glyph, 1, &mut width) };
        check()?;
        Ok(width)
    }
    fn metrics(&self) -> Result<d::FontMetrics> {
        let mut metrics = d::FontMetrics::default();
        reset();
        unsafe { d::OH_Drawing_FontGetMetrics(self.font.ptr(), &mut metrics) };
        check()?;
        Ok(metrics)
    }
    fn blob(&self, glyph: u16) -> Result<Owned<d::TextBlob>> {
        reset();
        Owned::new(
            unsafe {
                d::OH_Drawing_TextBlobCreateFromText(
                    (&glyph as *const u16).cast::<c_void>(),
                    2,
                    self.font.ptr(),
                    d::TEXT_GLYPH_ID,
                )
            },
            d::OH_Drawing_TextBlobDestroy,
        )
    }
    fn bounds(&self, glyph: u16) -> Result<[f32; 4]> {
        let blob = self.blob(glyph)?;
        reset();
        let rect = Owned::new(
            unsafe { d::OH_Drawing_RectCreate(0.0, 0.0, 0.0, 0.0) },
            d::OH_Drawing_RectDestroy,
        )?;
        reset();
        unsafe { d::OH_Drawing_TextBlobGetBounds(blob.ptr(), rect.ptr()) };
        check()?;
        Ok(unsafe {
            [
                d::OH_Drawing_RectGetLeft(rect.ptr()),
                d::OH_Drawing_RectGetTop(rect.ptr()),
                d::OH_Drawing_RectGetRight(rect.ptr()),
                d::OH_Drawing_RectGetBottom(rect.ptr()),
            ]
        })
    }
}

fn raster_bounds(params: &RenderGlyphParams) -> Result<Bounds<DevicePixels>> {
    let font = NativeFont::new(f32::from(params.font_size) * params.scale_factor)?;
    let glyph = u16::try_from(params.glyph_id.0)?;
    let [left, top, right, bottom] = font.bounds(glyph)?;
    if ![left, top, right, bottom].into_iter().all(f32::is_finite) {
        return Err(crate::gpui_core::private::anyhow::anyhow!(
            "invalid glyph bounds"
        ));
    }
    if right <= left || bottom <= top {
        return Ok(Bounds::default());
    }
    let shift_x = params.subpixel_variant.x as f32 / SUBPIXEL_VARIANTS_X as f32;
    let x0 = (left + shift_x).floor() as i32;
    let y0 = top.floor() as i32;
    let x1 = (right + shift_x).ceil() as i32;
    let y1 = bottom.ceil() as i32;
    Ok(Bounds::new(
        point(DevicePixels(x0), DevicePixels(y0)),
        size(DevicePixels(x1 - x0), DevicePixels(y1 - y0)),
    ))
}

fn rasterize(
    params: &RenderGlyphParams,
    bounds: Bounds<DevicePixels>,
) -> Result<(Size<DevicePixels>, Vec<u8>)> {
    let width = u32::try_from(bounds.size.width.0)?;
    let height = u32::try_from(bounds.size.height.0)?;
    if width == 0 || height == 0 {
        return Ok((bounds.size, Vec::new()));
    }
    if width > 4096 || height > 4096 {
        return Err(crate::gpui_core::private::anyhow::anyhow!(
            "glyph bitmap too large"
        ));
    }
    let font = NativeFont::new(f32::from(params.font_size) * params.scale_factor)?;
    let blob = font.blob(u16::try_from(params.glyph_id.0)?)?;
    reset();
    let bitmap = Owned::new(
        unsafe { d::OH_Drawing_BitmapCreate() },
        d::OH_Drawing_BitmapDestroy,
    )?;
    let format = d::BitmapFormat {
        color_format: d::COLOR_ALPHA_8,
        alpha_format: d::ALPHA_PREMUL,
    };
    reset();
    unsafe { d::OH_Drawing_BitmapBuild(bitmap.ptr(), width, height, &format) };
    check()?;
    reset();
    let canvas = Owned::new(
        unsafe { d::OH_Drawing_CanvasCreate() },
        d::OH_Drawing_CanvasDestroy,
    )?;
    reset();
    unsafe {
        d::OH_Drawing_CanvasBind(canvas.ptr(), bitmap.ptr());
        d::OH_Drawing_CanvasClear(canvas.ptr(), 0);
    }
    check()?;
    reset();
    let brush = Owned::new(
        unsafe { d::OH_Drawing_BrushCreate() },
        d::OH_Drawing_BrushDestroy,
    )?;
    reset();
    unsafe {
        d::OH_Drawing_BrushSetColor(brush.ptr(), 0xffffffff);
        d::OH_Drawing_BrushSetAntiAlias(brush.ptr(), true);
        d::OH_Drawing_CanvasAttachBrush(canvas.ptr(), brush.ptr());
    }
    check()?;
    let shift_x = params.subpixel_variant.x as f32 / SUBPIXEL_VARIANTS_X as f32;
    reset();
    unsafe {
        d::OH_Drawing_CanvasDrawTextBlob(
            canvas.ptr(),
            blob.ptr(),
            shift_x - bounds.origin.x.0 as f32,
            -(bounds.origin.y.0 as f32),
        );
        d::OH_Drawing_CanvasDetachBrush(canvas.ptr());
    }
    check()?;
    let mut row_bytes = 0u32;
    d::check_result(unsafe { d::OH_Drawing_BitmapGetRowBytes(bitmap.ptr(), &mut row_bytes) })
        .map_err(|error| {
            crate::gpui_core::private::anyhow::anyhow!("bitmap stride error {}", error.0)
        })?;
    if row_bytes < width {
        return Err(crate::gpui_core::private::anyhow::anyhow!(
            "glyph bitmap stride too short"
        ));
    }
    let pixels = unsafe { d::OH_Drawing_BitmapGetPixels(bitmap.ptr()) };
    if pixels.is_null() {
        return Err(crate::gpui_core::private::anyhow::anyhow!(
            "glyph bitmap has no pixels"
        ));
    }
    let mut alpha = Vec::with_capacity((width * height) as usize);
    for row in 0..height as usize {
        let start = row * row_bytes as usize;
        alpha.extend_from_slice(unsafe {
            std::slice::from_raw_parts(pixels.add(start), width as usize)
        });
    }
    Ok((bounds.size, alpha))
}

impl PlatformTextSystem for OhosTextSystem {
    fn add_fonts(&self, _fonts: Vec<Cow<'static, [u8]>>) -> Result<()> {
        Err(crate::gpui_core::private::anyhow::anyhow!(
            "custom fonts unsupported"
        ))
    }
    fn all_font_names(&self) -> Vec<String> {
        vec![".SystemUIFont".into()]
    }
    fn font_id(&self, _descriptor: &Font) -> Result<FontId> {
        Ok(FontId(1))
    }
    fn font_metrics(&self, _font_id: FontId) -> FontMetrics {
        NativeFont::new(1000.0)
            .and_then(|font| font.metrics())
            .map(|m| FontMetrics {
                units_per_em: 1000,
                ascent: -m.ascent,
                descent: -m.descent,
                line_gap: m.leading,
                underline_position: -m.underline_position,
                underline_thickness: m.underline_thickness,
                cap_height: -m.cap_height,
                x_height: -m.x_height,
                bounding_box: Bounds::new(
                    point(m.x_min, -m.bottom),
                    size(m.x_max - m.x_min, m.bottom - m.top),
                ),
            })
            .unwrap_or(FontMetrics {
                units_per_em: 1000,
                ascent: 800.0,
                descent: -200.0,
                line_gap: 0.0,
                underline_position: -100.0,
                underline_thickness: 50.0,
                cap_height: 700.0,
                x_height: 500.0,
                bounding_box: Bounds::default(),
            })
    }
    fn typographic_bounds(&self, _font_id: FontId, glyph_id: GlyphId) -> Result<Bounds<f32>> {
        let [l, t, r, b] = NativeFont::new(1000.0)?.bounds(u16::try_from(glyph_id.0)?)?;
        Ok(Bounds::new(point(l, -b), size(r - l, b - t)))
    }
    fn advance(&self, _font_id: FontId, glyph_id: GlyphId) -> Result<Size<f32>> {
        Ok(size(
            NativeFont::new(1000.0)?.width(u16::try_from(glyph_id.0)?)?,
            0.0,
        ))
    }
    fn glyph_for_char(&self, _font_id: FontId, ch: char) -> Option<GlyphId> {
        NativeFont::new(1000.0)
            .and_then(|font| font.glyph(ch))
            .ok()
            .flatten()
            .map(|id| GlyphId(id as u32))
    }
    fn glyph_raster_bounds(&self, params: &RenderGlyphParams) -> Result<Bounds<DevicePixels>> {
        raster_bounds(params)
    }
    fn rasterize_glyph(
        &self,
        params: &RenderGlyphParams,
        bounds: Bounds<DevicePixels>,
    ) -> Result<(Size<DevicePixels>, Vec<u8>)> {
        rasterize(params, bounds)
    }
    fn layout_line(&self, text: &str, font_size: Pixels, runs: &[FontRun]) -> LineLayout {
        let metrics = self.font_metrics(FontId(1));
        let mut width = px(0.0);
        let mut shaped = Vec::new();
        let mut offset = 0usize;
        for run in runs {
            let end = (offset + run.len).min(text.len());
            let Some(part) = text.get(offset..end) else {
                break;
            };
            let mut glyphs = Vec::new();
            for (index, ch) in part.char_indices() {
                if let Some(id) = self.glyph_for_char(run.font_id, ch) {
                    glyphs.push(ShapedGlyph {
                        id,
                        position: point(width, px(0.0)),
                        index: offset + index,
                        is_emoji: false,
                    });
                    if let Ok(advance) = self.advance(run.font_id, id) {
                        width += font_size * advance.width / 1000.0;
                    }
                }
            }
            if !glyphs.is_empty() {
                shaped.push(ShapedRun {
                    font_id: run.font_id,
                    glyphs,
                });
            }
            offset = end;
        }
        LineLayout {
            font_size,
            width,
            ascent: font_size * metrics.ascent / 1000.0,
            descent: font_size * metrics.descent / 1000.0,
            runs: shaped,
            len: text.len(),
        }
    }
    fn recommended_rendering_mode(
        &self,
        _font_id: FontId,
        _font_size: Pixels,
    ) -> TextRenderingMode {
        TextRenderingMode::Grayscale
    }
}
