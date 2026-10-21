//! Minimal Node-API ABI from the API 26 SDK's `node_api.h` and
//! `js_native_api_types.h`. Declarations used by the probe and XComponent host.

use std::ffi::{c_char, c_int, c_uint, c_void};

pub type Env = *mut c_void;
pub type Value = *mut c_void;
pub type CallbackInfo = *mut c_void;
pub type Callback = unsafe extern "C" fn(Env, CallbackInfo) -> Value;
pub type AddonRegister = unsafe extern "C" fn(Env, Value) -> Value;

pub const OK: c_int = 0;
pub const DEFAULT: c_int = 0;

#[repr(C)]
pub struct PropertyDescriptor {
    pub utf8name: *const c_char,
    pub name: Value,
    pub method: Option<Callback>,
    pub getter: Option<Callback>,
    pub setter: Option<Callback>,
    pub value: Value,
    pub attributes: c_int,
    pub data: *mut c_void,
}

#[repr(C)]
pub struct Module {
    pub nm_version: c_int,
    pub nm_flags: c_uint,
    pub nm_filename: *const c_char,
    pub nm_register_func: Option<AddonRegister>,
    pub nm_modname: *const c_char,
    pub nm_priv: *mut c_void,
    pub reserved: [*mut c_void; 4],
}

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(std::mem::size_of::<Module>() == 72);
    assert!(std::mem::align_of::<Module>() == 8);
    assert!(std::mem::offset_of!(Module, nm_register_func) == 16);
    assert!(std::mem::offset_of!(Module, reserved) == 40);
    assert!(std::mem::size_of::<PropertyDescriptor>() == 64);
    assert!(std::mem::align_of::<PropertyDescriptor>() == 8);
    assert!(std::mem::offset_of!(PropertyDescriptor, method) == 16);
    assert!(std::mem::offset_of!(PropertyDescriptor, attributes) == 48);
};

#[link(name = "ace_napi.z")]
unsafe extern "C" {
    pub fn napi_module_register(module: *mut Module);
    pub fn napi_create_uint32(env: Env, value: u32, result: *mut Value) -> c_int;
    pub fn napi_define_properties(
        env: Env,
        object: Value,
        property_count: usize,
        properties: *const PropertyDescriptor,
    ) -> c_int;
    pub fn napi_get_named_property(
        env: Env,
        object: Value,
        utf8name: *const c_char,
        result: *mut Value,
    ) -> c_int;
    pub fn napi_unwrap(env: Env, object: Value, result: *mut *mut c_void) -> c_int;
}
