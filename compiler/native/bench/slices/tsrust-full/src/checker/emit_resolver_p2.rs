//! Go `checker/emitresolver.go` lines 601 to 1298 (`requiresAddingImplicitUndefinedWorker`
//! to the end). The `EmitResolver` struct, its links and `newEmitResolver` are in
//! `emit_resolver_p1.rs`.
//!
//! PORT: Go methods read `r.checker`. A Rust resolver cannot hold `&mut Checker`, so:
//! - A Go unexported method takes the checker as an explicit `c: &mut Checker`
//!   parameter (`c` is Go `r.checker`).
//! - A Go exported method has the `crate::printer::EmitResolver` trait signature (the
//!   trait impl in `emit_resolver_p1.rs` calls it). A method that locks
//!   (`r.checkerMu.Lock(); defer r.checkerMu.Unlock()`) gets the checker with
//!   `self.with_checker(|c| ...)`. An exported method that Go does not lock also uses
//!   `with_checker`, because the trait signature has no checker, and has a `_worker`
//!   twin that takes `c` for callers that already hold the checker.
//! - Go exported and unexported methods with the same snake name: the exported one has
//!   the `_exported` suffix (PORTING.md).

use crate::prelude::*;

use crate::binder::reference_resolver::{
    
};
use crate::checker::emit_resolver_p1::{};
use crate::printer::{
    SymbolAccessibility, SymbolAccessibilityResult, TypeReferenceSerializationKind,
};


// Go: checker/emitresolver.go:693 isConstEnumOrConstEnumOnlyModule
// PORT: already ported as `Checker::is_const_enum_or_const_enum_only_module` in extras.rs.


// Go: checker/emitresolver.go:804 MarkLinkedReferencesRecursively.func1 (visit)
// PORT: the Go closure is recursive (it passes itself to `ForEachChild`), so it is a named fn.

// PORT: Go passes `*printer.EmitContext` to `NewNodeBuilder`, which keeps it. The Rust node
// builder keeps an `Rc<EmitContext>`; the context's factory gives it.

