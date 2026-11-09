//! PORT: not in Go. The emit resolver of the JS part of a file that runs on
//! the program's emit pool (`program_emit`), where there is no checker.
//! `emitter::js_emit_needs_checker` sends a JS part there only when its
//! transforms make no emit resolver call. Every method panics, so a call
//! that the rule misses ends the run (exit 70). It can never give an output
//! that differs from the checker thread's.

use crate::prelude::*;

/// An emit resolver whose every method panics (see the file comment).
pub struct NoCheckerEmitResolver;

/// The panic of every `NoCheckerEmitResolver` method.
fn no_checker(method: &str) -> ! {
    panic!("JS emit off the checker thread called EmitResolver::{method}");
}

