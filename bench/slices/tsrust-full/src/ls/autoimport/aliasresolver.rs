use crate::ls::autoimport::prelude::*;

// Port of Go `ls/autoimport/aliasresolver.go`.
//
// PORT: Go `aliasResolver` implements `checker.Program`, so the registry can
// make a checker (`checker.NewChecker(aliasResolver, nil)`) over node_modules
// entrypoint files that are not in any program. A ts_goport checker reads
// its program version (`prog()`), so `new_checker` makes an alias resolver
// program (`program::new_alias_resolver_program`). The `checker.Program`
// methods are inherent methods with the Go logic. The `program.rs`
// functions call the lazy ones through `program::AliasResolverProgram`, and
// give Go's constant or panic for the others.
// PORT: the registry host parses the files outside any program. Each file
// is published with no program and bound into the binder lineage, as
// sourcedefinition does (`bind_alias_resolver_source_file`).

use crate::frontend::core_ext::HasFileName;
use crate::frontend::module;
use crate::frontend::packagejson;
use crate::frontend::tsoptions;
use crate::frontend::tspath;
use crate::program::{};
use std::cell::Cell;
use std::sync::Arc;

// Go: ls/autoimport/aliasresolver.go:16 pathAndFileName
#[derive(Clone, Debug, Default)]
pub struct PathAndFileName {
    pub path: tspath::Path,
    pub file_name: String,
}

// Go: ls/autoimport/aliasresolver.go:21 aliasResolver
// PORT: Go `collections.SyncMap` values are `RefCell<FxHashMap>` (one
// thread). A Go nil `symlinks` map is an empty map. The fields after
// `resolved_modules` are the port's (see `new_checker` and
// `new_narrow_checker`).

// Go: ls/autoimport/aliasresolver.go:33 newAliasResolver




// Go: parser/references.go:24 collectModuleReferences
// PORT: without the ambient module conditions (see
// `checker_module_references`). Go checker `resolveExternalModuleNameWorker`
// takes any string literal like specifier.
// PORT: `re_exports_only` keeps only the export declarations
// (`narrow_module_references`).



// Go: ls/autoimport/aliasresolver.go:241 var _ checker.Program = (*aliasResolver)(nil)
// PORT: the lazy `checker.Program` methods, which the alias resolver
// program calls (see the file header).
