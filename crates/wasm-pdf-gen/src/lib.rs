#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
pub mod pdf;

#[cfg(any(target_os = "wasi", not(target_arch = "wasm32")))]
pub mod files;
