//! Port of Go `internal/api/module_resolution.go` (ts#64299): module
//! resolution overrides for API programs.
//!
//! PORT: Go `module.Resolver` is an interface at N (the program lane ports it
//! as the trait `module::Resolver`); a resolver value is
//! `Rc<dyn module::Resolver>`. Go `project.ModuleResolverFactory` is the
//! trait `project::ModuleResolverFactory`. The api lane notes for the
//! program and server lanes give the exact signatures this file calls.
//!
//! PORT: Go keeps `session *Session` in the factory to register and release
//! program resolution contexts. A Rust `&Session` cannot be kept, so the
//! registry of contexts is a shared `Rc<ProgramResolutionContexts>` that the
//! session and the factories hold.

use crate::api::prelude::*;

use crate::api::proto;
use crate::frontend::json_ext::AnyValue;
use crate::frontend::module;
use crate::gostd::errors;
use crate::program::PackageId as ModulePackageId;
// PORT: the explicit import picks Go `module.ResolvedModule` over the api
// `proto::ResolvedModule` of the api prelude.
use std::cell::Cell;
use std::sync::Arc;

// Go: api/session.go moduleResolverRegistration (ts#64299)

// Go: api/module_resolution.go moduleResolverFactory

// Go: api/module_resolution.go programResolutionContext
// PORT: Go `mu` is not ported (one thread).


// Go: api/module_resolution.go callbackModuleResolver

// Go: api/module_resolution.go moduleResolverFactory.NewResolver

// Go: api/module_resolution.go callbackModuleResolver (module.Resolver)


// Go: api/module_resolution.go compileModuleResolutionSpec

// Go: api/module_resolution.go staticModuleResolutionToResolvedModule

// Go: api/module_resolution.go moduleResolutionTraceToStrings




// Go: api/module_resolution.go moduleResolutionError



