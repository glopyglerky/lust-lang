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

/// Go passes the api `Session` (`FS`, `GetCurrentDirectory`) or a
/// `*project.Snapshot` as a `module.ResolutionHost`.
// PORT: the Rust trait returns borrowed values, so this host keeps the file
// system and directory that the Go methods return at the call.
struct ApiResolutionHost {
    fs: Rc<dyn vfs::Fs>,
    current_directory: String,
}

impl module::ResolutionHost for ApiResolutionHost {
    fn fs(&self) -> &dyn vfs::Fs {
        &*self.fs
    }

    fn get_current_directory(&self) -> &str {
        &self.current_directory
    }
}

