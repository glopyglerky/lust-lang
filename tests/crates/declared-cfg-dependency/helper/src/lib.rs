#![no_std]
#[cfg(checked_build)]
compile_error!("workspace declaration is not an enabled flag");
pub fn enabled() -> bool { cfg!(checked_build) }
