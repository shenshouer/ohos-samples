//! GPU context and on-screen Surface interfaces from `native_drawing/*.h`.

use core::ffi::c_void;

use super::paint::Canvas;

pub const COLOR_RGBA_8888: i32 = 4;
pub const ALPHA_PREMUL: i32 = 2;

#[repr(C)]
pub struct GpuContext {
    _private: [u8; 0],
}

#[repr(C)]
pub struct Surface {
    _private: [u8; 0],
}

/// `OH_Drawing_Image_Info`. The C enum fields are 32-bit values on the
/// aarch64 OHOS SDK; keeping them as integers also preserves unknown values.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ImageInfo {
    pub width: i32,
    pub height: i32,
    pub color_type: i32,
    pub alpha_type: i32,
}

#[link(name = "native_drawing")]
unsafe extern "C" {
    // drawing_gpu_context.h; create introduced in API 16, destroy in API 12.
    pub fn OH_Drawing_GpuContextCreate() -> *mut GpuContext;
    pub fn OH_Drawing_GpuContextDestroy(context: *mut GpuContext);

    // drawing_surface.h; on-screen create and flush introduced in API 16.
    pub fn OH_Drawing_SurfaceCreateOnScreen(
        context: *mut GpuContext,
        info: ImageInfo,
        window: *mut c_void,
    ) -> *mut Surface;
    pub fn OH_Drawing_SurfaceGetCanvas(surface: *mut Surface) -> *mut Canvas;
    pub fn OH_Drawing_SurfaceFlush(surface: *mut Surface) -> i32;
    pub fn OH_Drawing_SurfaceDestroy(surface: *mut Surface);

}

const _: () = {
    use core::mem::{align_of, offset_of, size_of};
    assert!(size_of::<ImageInfo>() == 16);
    assert!(align_of::<ImageInfo>() == 4);
    assert!(offset_of!(ImageInfo, color_type) == 8);
};
