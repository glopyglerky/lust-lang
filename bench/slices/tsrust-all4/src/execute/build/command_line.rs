use crate::frontend::prelude::*;
use std::cell::OnceCell;

// This file ports the build mode (`tsc -b`) command line pieces:
// core/buildoptions.go, tsoptions/parsedbuildcommandline.go,
// tsoptions/commandlineparser.go ParseBuildCommandLine, and the
// buildOptionsParser / ParseBuildOptions parts of tsoptions/parsinghelpers.go
// that the frontend port left out.

// Go: core/buildoptions.go:3 BuildOptions
// PORT: Go `*int` (64-bit) is `Option<i64>`. The `noCopy` marker is dropped.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BuildOptions {
    pub dry: Tristate,
    pub force: Tristate,
    pub verbose: Tristate,
    pub builders: Option<i64>,
    pub stop_build_on_errors: Tristate,

    // CompilerOptions are not parsed here and will be available on ParsedBuildCommandLine

    // Internal fields
    pub clean: Tristate,
}

// Go: tsoptions/parsinghelpers.go:252 buildOptionsParser
// PORT: the embedded Go `*core.BuildOptions` is a mutable borrow, as in
// `CompilerOptionsParser`.
pub struct BuildOptionsParser<'a> {
    pub build_options: &'a mut BuildOptions,
}


// Go: tsoptions/parsinghelpers.go:626 ParseBuildOptions
// PORT: Go `allOptions` can be nil; the Rust caller always has options, so
// that nil check is dropped (as in `parse_compiler_options`).

// Go: tsoptions/parsedbuildcommandline.go:12 ParsedBuildCommandLine
// PORT: Go `*core.CompilerOptions` is `Rc<CompilerOptions>`, as in
// `ParsedCommandLine`. Go `Raw any` holds the parser's
// `*collections.OrderedMap`, which is `CompilerOptionsValue::Map`.
// `sync.Once` plus the cached field is a `OnceCell` (for both
// `resolvedProjectPaths` and `locale`).
// PORT: Go also has `WatchOptions`. Watch mode is out of scope and the crate
// has no `WatchOptions`, so it is left out. It does not change build output.


// Go: tsoptions/commandlineparser.go:64 ParseBuildCommandLine
// PORT: Go nil `commandLine` is an empty slice here already. The
// `WatchOptions` conversion is left out (see `ParsedBuildCommandLine`).

// Go pointer equality on `*CommandLineOption`, where nil equals nil.
