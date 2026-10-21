//! Link-only probe for the two SDK libraries used by `sdk_ffi`.
//! Build for aarch64 OHOS; do not run this binary on the device.
#![allow(dead_code)]

#[path = "../src/sdk_ffi/mod.rs"]
mod sdk_ffi;

use sdk_ffi::{drawing as d, xcomponent as x};

fn main() {
    let symbols = [
        x::OH_NativeXComponent_RegisterCallback as *const (),
        x::OH_NativeXComponent_GetXComponentSize as *const (),
        x::OH_NativeXComponent_GetTouchEvent as *const (),
        d::OH_Drawing_GpuContextCreate as *const (),
        d::OH_Drawing_GpuContextDestroy as *const (),
        d::OH_Drawing_SurfaceCreateOnScreen as *const (),
        d::OH_Drawing_SurfaceGetCanvas as *const (),
        d::OH_Drawing_SurfaceFlush as *const (),
        d::OH_Drawing_SurfaceDestroy as *const (),
        d::OH_Drawing_ErrorCodeGet as *const (),
        d::OH_Drawing_ErrorCodeReset as *const (),
        d::OH_Drawing_CanvasClear as *const (),
        d::OH_Drawing_CanvasAttachBrush as *const (),
        d::OH_Drawing_CanvasDetachBrush as *const (),
        d::OH_Drawing_CanvasDrawRect as *const (),
        d::OH_Drawing_CanvasDrawRoundRect as *const (),
        d::OH_Drawing_BrushCreate as *const (),
        d::OH_Drawing_BrushSetColor as *const (),
        d::OH_Drawing_BrushDestroy as *const (),
        d::OH_Drawing_RectCreate as *const (),
        d::OH_Drawing_RectDestroy as *const (),
        d::OH_Drawing_RoundRectCreate as *const (),
        d::OH_Drawing_RoundRectDestroy as *const (),
    ];
    std::hint::black_box(symbols);
}
