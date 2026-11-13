// Lust `std` platform crate (docs/ARCH.md sections 3.1 and 3.7,
// package W-SHIM): the OS-facing std modules in source.
//
// The resolver loads this crate as a lust-owned dependency named `std`
// when a file of the session can name a platform module
// (`resolve::imports::STD_PLATFORM_MODULES`). A written
// `std::io::Write`, `std::env::var`, or `use std::io::{self, Write}`
// then resolves into the modules below when the module defines the
// item; other std paths keep the compiler's std handling. Bodies call
// the runtime through `core::intrinsics` (rows in src/ty/lang.rs, C in
// src/runtime/sys.c). The shipped-crate privilege is matched by this
// crate's files, never by a name user code can write.
//
// Rules for this source: valid Rust, only features lust supports, and
// the same observable behavior as rustc 1.99 std on Linux through the
// public API (texts, exit codes, Debug forms). Items std marks
// unstable carry a `__` name prefix here; user code cannot name them
// in rustc either.

pub mod io;
pub mod ffi;
pub mod path;
pub mod env;
pub mod fs;
pub mod os;
#[path = "process_os.rs"]
pub mod process;
