//! Minimal hand-written API 26 Native SDK declarations for the mobile counter.
//!
//! Modules follow SDK capabilities. Raw declarations stay here; the host and
//! renderer own callback lifetimes and native resources. NativeWindow's tiny
//! query helpers also live beside its variadic C ABI declaration.

pub(crate) mod display;
pub mod drawing;
pub mod hilog;
pub mod napi;
pub mod native_window;
pub mod xcomponent;
