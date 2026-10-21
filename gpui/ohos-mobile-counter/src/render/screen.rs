//! Native Drawing Surface ownership for the current XComponent window.

use std::ffi::c_void;

use crate::gpui::atlas::MonoAtlas;
use crate::gpui_core::Scene;
use crate::sdk_ffi::drawing as d;

#[derive(Debug)]
pub struct ScreenError {
    pub step: &'static str,
    pub code: i32,
}

fn status(step: &'static str) -> Result<(), ScreenError> {
    let code = unsafe { d::OH_Drawing_ErrorCodeGet() };
    d::check_result(code).map_err(|error| ScreenError {
        step,
        code: error.0,
    })
}

fn null_pointer(step: &'static str) -> ScreenError {
    let code = unsafe { d::OH_Drawing_ErrorCodeGet() };
    ScreenError {
        step,
        code: if code == d::SUCCESS { -1 } else { code },
    }
}

pub struct ScreenDrawing {
    context: usize,
    surface: usize,
}

impl Drop for ScreenDrawing {
    fn drop(&mut self) {
        // SAFETY: the on-screen Surface depends on the owned GPU context.
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

impl ScreenDrawing {
    pub fn create(window: *mut c_void, width: i32, height: i32) -> Result<Self, ScreenError> {
        if window.is_null() || width <= 0 || height <= 0 {
            return Err(ScreenError {
                step: "image_info",
                code: 401,
            });
        }
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
        unsafe { d::OH_Drawing_ErrorCodeReset() };
        let surface = unsafe { d::OH_Drawing_SurfaceCreateOnScreen(context, info, window) };
        owned.surface = surface as usize;
        if surface.is_null() {
            return Err(null_pointer("surface_create_on_screen"));
        }
        status("surface_create_on_screen")?;
        Ok(owned)
    }

    pub(crate) fn paint_scene(
        &mut self,
        scene: &Scene,
        atlas: &MonoAtlas,
    ) -> crate::gpui_core::Result<crate::render::scene::PaintStats> {
        unsafe { d::OH_Drawing_ErrorCodeReset() };
        let canvas = unsafe { d::OH_Drawing_SurfaceGetCanvas(self.surface as *mut d::Surface) };
        if canvas.is_null() {
            return Err(crate::gpui_core::private::anyhow::anyhow!(
                "scene surface_get_canvas returned null"
            ));
        }
        status("surface_get_canvas").map_err(|error| {
            crate::gpui_core::private::anyhow::anyhow!(
                "scene {} failed: {}",
                error.step,
                error.code
            )
        })?;
        let stats = crate::render::scene::paint(canvas, scene, atlas)?;
        unsafe { d::OH_Drawing_ErrorCodeReset() };
        let code = unsafe { d::OH_Drawing_SurfaceFlush(self.surface as *mut d::Surface) };
        d::check_result(code).map_err(|error| {
            crate::gpui_core::private::anyhow::anyhow!("scene surface_flush failed: {}", error.0)
        })?;
        status("surface_flush").map_err(|error| {
            crate::gpui_core::private::anyhow::anyhow!(
                "scene {} failed: {}",
                error.step,
                error.code
            )
        })?;
        Ok(stats)
    }
}
