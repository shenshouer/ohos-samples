//! HarmonyOS mobile counter: host adapters, SDK bindings and probe renderer.

#[cfg(target_env = "ohos")]
mod host;
#[cfg(target_env = "ohos")]
mod render;
#[cfg(target_env = "ohos")]
pub mod sdk_ffi;

/// Bit 0: Rust entry reached; bit 1: Native Drawing call succeeded;
/// bit 2: the XComponent SDK symbol was resolved by the loader.
#[cfg(target_env = "ohos")]
#[unsafe(no_mangle)]
pub extern "C" fn gpui_mobile_probe() -> u32 {
    use sdk_ffi::{drawing, xcomponent};

    let mut result = 1;
    // SAFETY: The error-code functions have no pointer parameters and are
    // documented for this SDK level. The module is loaded before this call.
    unsafe {
        drawing::OH_Drawing_ErrorCodeReset();
        if drawing::OH_Drawing_ErrorCodeGet() == drawing::SUCCESS {
            result |= 1 << 1;
        }
    }
    let callback_symbol =
        std::hint::black_box(xcomponent::OH_NativeXComponent_RegisterCallback as *const ());
    if !callback_symbol.is_null() {
        result |= 1 << 2;
    }
    result
}
