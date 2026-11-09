//! Go `binder/referenceresolver.go`.
//!
//! PORT: Go `ReferenceResolver` is an interface with one implementation
//! (`referenceResolver`). Here it is the struct `ReferenceResolver`.
//! Go hook closures capture the checker. Here each hook is a plain function
//! that takes the checker as its first argument, and each resolver method
//! takes `c: &mut H`, a `NameResolverHost`, to pass to the hooks and to the
//! fallback name resolver (`crate::checker::NameResolver`). With hooks, the
//! host is the checker (the checker's emit resolver). With no hooks, the host
//! can be the program's binder symbols (JS emit, see
//! `transformers::reference_resolver`), since Go then reads only binder
//! symbols.

use crate::prelude::*;

use std::cell::Cell;


// Go: binder/referenceresolver.go:18 ReferenceResolverHooks

// Go: binder/referenceresolver.go:31 referenceResolver

// Go: binder/referenceresolver.go:37 NewReferenceResolver

