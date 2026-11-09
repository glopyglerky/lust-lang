// `core::ops` overlay (docs/ARCH.md section 3.1; W-STD-C): `ControlFlow`.
// The other `core::ops` items (the operator traits, ranges, `Deref`, the
// `Fn` traits) stay compiler-known; `std::ops::X` and `core::ops::X` walk
// into this module only for the names it defines (`STD_MODULES` in
// src/stdmods.rs).
//
// Lust std source, adapted from rust-lang/rust 1.99
// `core/src/ops/control_flow.rs`. The `?` operator on a `ControlFlow` is
// the compiler's (`try_expr` in src/check/mod.rs, `try_into` in
// src/lower.rs): `Break(b)` returns `ControlFlow::Break(b)` from the
// enclosing fn or closure, with `b` unconverted, as std's
// `FromResidual<ControlFlow<B, Infallible>>`. Not here: the unstable
// methods (`break_ok`, `continue_ok`).

/// Tells an operation whether it should exit early or go on as usual.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ControlFlow<B, C = ()> {
    /// Move on to the next phase of the operation as normal.
    Continue(C),
    /// Exit the operation without running subsequent phases.
    Break(B),
}

impl<B, C> ControlFlow<B, C> {
    pub fn is_break(&self) -> bool {
        matches!(*self, ControlFlow::Break(_))
    }

    pub fn is_continue(&self) -> bool {
        matches!(*self, ControlFlow::Continue(_))
    }

    pub fn break_value(self) -> Option<B> {
        match self {
            ControlFlow::Continue(..) => None,
            ControlFlow::Break(x) => Some(x),
        }
    }

    pub fn continue_value(self) -> Option<C> {
        match self {
            ControlFlow::Continue(x) => Some(x),
            ControlFlow::Break(..) => None,
        }
    }

    pub fn map_break<T, F: FnOnce(B) -> T>(self, f: F) -> ControlFlow<T, C> {
        match self {
            ControlFlow::Continue(x) => ControlFlow::Continue(x),
            ControlFlow::Break(x) => ControlFlow::Break(f(x)),
        }
    }

    pub fn map_continue<T, F: FnOnce(C) -> T>(self, f: F) -> ControlFlow<B, T> {
        match self {
            ControlFlow::Continue(x) => ControlFlow::Continue(f(x)),
            ControlFlow::Break(x) => ControlFlow::Break(x),
        }
    }
}
