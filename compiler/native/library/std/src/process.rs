// The `process` module of std: process exit (docs/ARCH.md section 3.1). Its
// text is spliced into user source, so it names no `std::` platform module
// path: the platform crate scan (`mentions_std_platform`) would load for it.
//
// Lust std source. The std loader (src/stdload.rs) parses and checks this
// file as part of the shipped `std` crate, then splices its fns into a
// user compile under mangled names (see `STD_FNS` in src/ty/lang.rs).
// This file is never fed to rustc as a replacement std; it only has to
// behave the same through the public API (section 3.1).

/// Terminates the current process with the given exit code.
/// Destructors are not run, as in real std. Stdout is flushed first,
/// because lust print output is buffered (the intrinsic does this).
pub fn exit(code: i32) -> ! {
    core::intrinsics::exit(code)
}
