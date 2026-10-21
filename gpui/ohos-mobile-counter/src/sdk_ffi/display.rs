//! Minimal declarations from SDK `window_manager/oh_display_manager.h` (API 12+).

#[link(name = "native_display_manager")]
unsafe extern "C" {
    fn OH_NativeDisplayManager_GetDefaultDisplayId(id: *mut u64) -> i32;
    fn OH_NativeDisplayManager_GetDefaultDisplayWidth(width: *mut i32) -> i32;
    fn OH_NativeDisplayManager_GetDefaultDisplayHeight(height: *mut i32) -> i32;
    fn OH_NativeDisplayManager_GetDefaultDisplayDensityPixels(density: *mut f32) -> i32;
    fn OH_NativeDisplayManager_GetDefaultDisplayRefreshRate(rate: *mut u32) -> i32;
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct DisplayInfo {
    pub(crate) id: u64,
    pub(crate) width_pixels: i32,
    pub(crate) height_pixels: i32,
    pub(crate) density: f32,
    pub(crate) refresh_hz: Option<u32>,
}

pub(crate) fn default_display() -> Option<DisplayInfo> {
    let (mut id, mut width, mut height, mut density, mut rate) = (0, 0, 0, 0.0, 0);
    // SAFETY: each SDK function writes to one valid scalar out-parameter.
    let statuses = unsafe {
        (
            OH_NativeDisplayManager_GetDefaultDisplayId(&mut id),
            OH_NativeDisplayManager_GetDefaultDisplayWidth(&mut width),
            OH_NativeDisplayManager_GetDefaultDisplayHeight(&mut height),
            OH_NativeDisplayManager_GetDefaultDisplayDensityPixels(&mut density),
        )
    };
    if statuses != (0, 0, 0, 0)
        || width <= 0
        || height <= 0
        || !density.is_finite()
        || density <= 0.0
    {
        return None;
    }
    // SAFETY: the SDK writes one valid uint32 out-parameter.
    let refresh_hz = (unsafe { OH_NativeDisplayManager_GetDefaultDisplayRefreshRate(&mut rate) }
        == 0
        && rate > 0)
        .then_some(rate);
    Some(DisplayInfo {
        id,
        width_pixels: width,
        height_pixels: height,
        density,
        refresh_hz,
    })
}
