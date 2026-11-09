use crate::frontend::prelude::*;
use std::sync::LazyLock;

// ---------------------------------------------------------------------------
// tsoptions/errors.go
// ---------------------------------------------------------------------------

// Go: tsoptions/errors.go:14 createDiagnosticForInvalidEnumType

// Go: tsoptions/errors.go:21 formatEnumTypeKeys

// Go: tsoptions/errors.go:28 getCompilerOptionValueTypeString


// createUnknownOptionError creates a diagnostic for an unknown option. If
// unknownDidYouMeanDiagnostic and optionsNameMap are provided, it also checks
// for a spelling suggestion and emits a "did you mean" diagnostic instead.
// Go: tsoptions/errors.go:60 createUnknownOptionError
// PORT: Go nil `unknownDidYouMeanDiagnostic` and `optionsNameMap` are `None`.

// Go: tsoptions/errors.go:92 CreateDiagnosticForNodeInSourceFile

// Go: tsoptions/errors.go:96 CreateDiagnosticForNodeInSourceFileOrCompilerDiagnostic

// Go: tsoptions/errors.go:103 extraKeyDiagnostics

// Go: tsoptions/errors.go:118 extraKeyDidYouMeanDiagnostics

// ---------------------------------------------------------------------------
// tsoptions/diagnostics.go
// ---------------------------------------------------------------------------

// Go: tsoptions/diagnostics.go:9 DidYouMeanOptionsDiagnostics

// Go: tsoptions/diagnostics.go:16 AlternateModeDiagnostics

// Go: tsoptions/diagnostics.go:21 ParseCommandLineWorkerDiagnostics
// PORT: Go `optionsNameMap` and `optionsNameMapOnce` are never read or set
// in the pinned source. `options_name_map` stays `None` and the `sync.Once`
// is dropped.

// Go: tsoptions/diagnostics.go:28 CompilerOptionsDidYouMeanDiagnostics
// PORT: Go package-level vars are `LazyLock` statics.

// Go: tsoptions/diagnostics.go:30 getParseCommandLineWorkerDiagnostics

// Go: tsoptions/diagnostics.go:46 watchOptionsDidYouMeanDiagnostics

// Go: tsoptions/diagnostics.go:56 buildOptionsDidYouMeanDiagnostics
