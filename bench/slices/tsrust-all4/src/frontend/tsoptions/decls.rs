use crate::frontend::prelude::*;
use std::sync::LazyLock;

// This file ports tsoptions/declscompiler.go, namemap.go, declsbuild.go,
// declswatch.go and declstypeacquisition.go.
// PORT: Go package-level vars are `LazyLock` statics. Declaration lists
// are `Vec<&'static CommandLineOption>` with leaked entries, so the same
// option has the same address in every list, as in Go.



// ---------------------------------------------------------------------------
// tsoptions/declscompiler.go
// ---------------------------------------------------------------------------

// Go: tsoptions/declscompiler.go:11 OptionsDeclarations

// Go: tsoptions/declscompiler.go:13 commonOptionsWithBuild

// Go: tsoptions/declscompiler.go:263 optionsForCompiler

// Go: tsoptions/declscompiler.go:1211 optionsType
// PORT: Go reads `core.CompilerOptions` fields by reflection. This returns
// the exported fields in Go declaration order, as (Go field name, value).
// The Go field index `i` is the index in this list (the unexported
// `noCopy` field is not in it).

// Go: tsoptions/declscompiler.go:1213 optionsHaveChanges

// Go: tsoptions/declscompiler.go:1234 ForEachCompilerOptionValue

// Go: tsoptions/declscompiler.go:1250 CompilerOptionsAffectSemanticDiagnostics

// Go: tsoptions/declscompiler.go:1259 CompilerOptionsAffectDeclarationPath

// Go: tsoptions/declscompiler.go:1268 CompilerOptionsAffectEmit

// ---------------------------------------------------------------------------
// tsoptions/namemap.go
// ---------------------------------------------------------------------------

// Go: tsoptions/namemap.go:9 CompilerNameMap
// Go: tsoptions/namemap.go:10 BuildNameMap
// Go: tsoptions/namemap.go:11 WatchNameMap

// Go: tsoptions/namemap.go:15 GetNameMapFromList

// Go: tsoptions/namemap.go:30 NameMap
// PORT: Go `OrderedMap.Set` on an existing key keeps the first position and
// replaces the value. `IndexMap::insert` does the same.


// ---------------------------------------------------------------------------
// tsoptions/declsbuild.go
// ---------------------------------------------------------------------------

// Go: tsoptions/declsbuild.go:9 TscBuildOption

// Go: tsoptions/declsbuild.go:19 OptionsForBuild

// Go: tsoptions/declsbuild.go:69 BuildOpts

// ---------------------------------------------------------------------------
// tsoptions/declswatch.go
// ---------------------------------------------------------------------------

// Go: tsoptions/declswatch.go:8 OptionsForWatch

// ---------------------------------------------------------------------------
// tsoptions/declstypeacquisition.go
// ---------------------------------------------------------------------------

// Go: tsoptions/declstypeacquisition.go:3 typeAcquisitionDeclaration

// Do not delete this without updating the website's tsconfig generation.
// Go: tsoptions/declstypeacquisition.go:10 typeAcquisitionDecls
