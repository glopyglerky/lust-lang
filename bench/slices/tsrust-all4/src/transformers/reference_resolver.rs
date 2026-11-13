//! The binder reference resolver as a `TransformReferenceResolver`.
//!
//! PORT: Go `getScriptTransformers` builds
//! `binder.NewReferenceResolver(options, binder.ReferenceResolverHooks{})`
//! for isolated modules. With no hooks, Go reads only the binder's
//! `ast.Symbol` values, which every goroutine shares. Here the resolver reads
//! the program's binder symbols (`BinderSymbols`), so it needs no checker and
//! runs on any thread: on the file's checker thread, or on the program's emit
//! pool (`emitter::program_emit`).

use crate::prelude::*;

use super::transformer::TransformReferenceResolver;
use crate::binder::reference_resolver::{
    
};





// Go: binder/referenceresolver.go:37 NewReferenceResolver
// PORT: Go `getScriptTransformers` passes empty hooks, so this takes none.
// A hook would need the checker, which `BinderSymbols` does not have.

