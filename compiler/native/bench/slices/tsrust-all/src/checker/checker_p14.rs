//! Port of typescript-go `internal/checker/checker.go` lines 12101-13024.
//!
//! PORT: Go `c.error(...)` returns the `*ast.Diagnostic` it already added, and
//! some Go callers mutate it afterwards (`AddRelatedInfo`). Our `Diagnostic`
//! is owned, so those callers build the diagnostic with
//! `new_diagnostic_for_node`, finish it, then call `add_diagnostic`. That is
//! exactly what Go `c.error` does, in the same order of effects.
//!
//! PORT: Go `func(left *Type, right *Type) bool` parameters become
//! `&mut dyn FnMut(&mut Checker, TypeId, TypeId) -> bool` (wrapped in
//! `Option` where Go passes nil).

use crate::prelude::*;

