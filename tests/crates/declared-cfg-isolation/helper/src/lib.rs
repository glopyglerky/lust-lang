#![no_std]

pub fn enabled() -> bool { cfg!(root_only) }
