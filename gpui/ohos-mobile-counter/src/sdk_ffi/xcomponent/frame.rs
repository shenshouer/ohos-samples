//! XComponent display frame callback (API 11, `libace_ndk.z.so`).

use super::XComponent;

pub type FrameCallback = unsafe extern "C" fn(*mut XComponent, u64, u64);

#[link(name = "ace_ndk.z")]
unsafe extern "C" {
    pub fn OH_NativeXComponent_RegisterOnFrameCallback(
        component: *mut XComponent,
        callback: Option<FrameCallback>,
    ) -> i32;
    pub fn OH_NativeXComponent_UnregisterOnFrameCallback(component: *mut XComponent) -> i32;
}
