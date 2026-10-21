//! Surface callbacks, component handle and size query (`libace_ndk.z.so`).

use core::ffi::c_void;

pub const RESULT_SUCCESS: i32 = 0;

#[repr(C)]
pub struct XComponent {
    _private: [u8; 0],
}

pub type SurfaceCallback = unsafe extern "C" fn(*mut XComponent, *mut c_void);

#[repr(C)]
pub struct Callbacks {
    pub on_surface_created: Option<SurfaceCallback>,
    pub on_surface_changed: Option<SurfaceCallback>,
    pub on_surface_destroyed: Option<SurfaceCallback>,
    pub dispatch_touch_event: Option<SurfaceCallback>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct XComponentError(pub i32);

pub fn check_result(code: i32) -> Result<(), XComponentError> {
    if code == RESULT_SUCCESS {
        Ok(())
    } else {
        Err(XComponentError(code))
    }
}

#[link(name = "ace_ndk.z")]
unsafe extern "C" {
    pub fn OH_NativeXComponent_RegisterCallback(
        component: *mut XComponent,
        callback: *mut Callbacks,
    ) -> i32;
    pub fn OH_NativeXComponent_GetXComponentSize(
        component: *mut XComponent,
        window: *const c_void,
        width: *mut u64,
        height: *mut u64,
    ) -> i32;
}

const _: () = {
    use core::mem::{align_of, size_of};
    assert!(size_of::<Callbacks>() == 32);
    assert!(align_of::<Callbacks>() == 8);
};
