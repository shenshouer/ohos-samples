//! Node-API registration without a C/C++ runtime bridge.

use std::ffi::c_void;
use std::ptr::{null, null_mut};

use crate::gpui;
use crate::host::{frame, logging, surface};
use crate::sdk_ffi::napi::{CallbackInfo, Env, Module, PropertyDescriptor, Value};
use crate::sdk_ffi::{hilog, napi, xcomponent};

const MODULE_NAME: &str = "gpui_mobile_counter\0";
const PROBE_NAME: &str = "probe\0";
const GPUI_START_NAME: &str = "gpuiStart\0";
const GPUI_PUMP_NAME: &str = "gpuiPump\0";
const GPUI_PAUSE_NAME: &str = "gpuiPause\0";
const GPUI_RESUME_NAME: &str = "gpuiResume\0";
const XCOMPONENT_OBJECT_NAME: &str = "__NATIVE_XCOMPONENT_OBJ__\0";

unsafe extern "C" fn probe(env: Env, _info: CallbackInfo) -> Value {
    let mut value = null_mut();
    // SAFETY: `env` is supplied by Node-API and `value` points to writable
    // storage for the returned opaque handle.
    if unsafe { napi::napi_create_uint32(env, crate::gpui_mobile_probe(), &mut value) } != napi::OK
    {
        return null_mut();
    }
    value
}

unsafe extern "C" fn gpui_start(env: Env, _info: CallbackInfo) -> Value {
    uint32(env, gpui::start())
}

unsafe extern "C" fn gpui_pump(env: Env, _info: CallbackInfo) -> Value {
    uint32(env, gpui::pump())
}

unsafe extern "C" fn gpui_pause(env: Env, _info: CallbackInfo) -> Value {
    frame::pause();
    uint32(env, 1)
}

unsafe extern "C" fn gpui_resume(env: Env, _info: CallbackInfo) -> Value {
    frame::resume();
    uint32(env, 1)
}

fn uint32(env: Env, result: u32) -> Value {
    let mut value = null_mut();
    // SAFETY: Node-API owns env; value is writable for the duration of this call.
    if unsafe { napi::napi_create_uint32(env, result, &mut value) } == napi::OK {
        value
    } else {
        null_mut()
    }
}

unsafe extern "C" fn init(env: Env, exports: Value) -> Value {
    let mut wrapped = null_mut();
    // SAFETY: the SDK populates this property when initializing the module
    // for an XComponent with the matching libraryname.
    let property_status = unsafe {
        napi::napi_get_named_property(
            env,
            exports,
            XCOMPONENT_OBJECT_NAME.as_ptr().cast(),
            &mut wrapped,
        )
    };
    if property_status == napi::OK && !wrapped.is_null() {
        let mut native: *mut c_void = null_mut();
        // SAFETY: wrapped is a Node-API value obtained from this environment.
        let unwrap_status = unsafe { napi::napi_unwrap(env, wrapped, &mut native) };
        if unwrap_status == napi::OK && !native.is_null() {
            surface::register(native.cast::<xcomponent::XComponent>());
        } else {
            logging::log(
                hilog::LOG_WARN,
                format!("init: native XComponent unavailable unwrap_status={unwrap_status}"),
            );
        }
    } else {
        logging::log(
            hilog::LOG_WARN,
            format!("init: XComponent property unavailable status={property_status}"),
        );
    }
    let property = |name: &'static str, method| PropertyDescriptor {
        utf8name: name.as_ptr().cast(),
        name: null_mut(),
        method,
        getter: None,
        setter: None,
        value: null_mut(),
        attributes: napi::DEFAULT,
        data: null_mut(),
    };
    let properties = [
        property(PROBE_NAME, Some(probe)),
        property(GPUI_START_NAME, Some(gpui_start)),
        property(GPUI_PUMP_NAME, Some(gpui_pump)),
        property(GPUI_PAUSE_NAME, Some(gpui_pause)),
        property(GPUI_RESUME_NAME, Some(gpui_resume)),
    ];
    // SAFETY: `exports` and `env` are provided by Node-API. The descriptor
    // remains live until this synchronous call returns.
    if unsafe { napi::napi_define_properties(env, exports, properties.len(), properties.as_ptr()) }
        != napi::OK
    {
        return null_mut();
    }
    exports
}

static mut MODULE: Module = Module {
    nm_version: 1,
    nm_flags: 0,
    nm_filename: null(),
    nm_register_func: Some(init),
    nm_modname: MODULE_NAME.as_ptr().cast(),
    nm_priv: null_mut::<c_void>(),
    reserved: [null_mut(); 4],
};

extern "C" fn register_module() {
    // SAFETY: ELF runs this constructor when loading the shared object. The
    // static module is process-lifetime storage, as required by Node-API.
    unsafe { napi::napi_module_register(&raw mut MODULE) };
}

#[used]
#[unsafe(link_section = ".init_array")]
static REGISTER_MODULE: extern "C" fn() = register_module;
