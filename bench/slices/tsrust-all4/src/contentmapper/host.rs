//! Port of Go `internal/contentmapper/host.go` (tsgo#4712).
//!
//! PORT: Go returns the pointer error types (`*TransformError`, ...) as
//! `error`. Each Rust error type is a value with `to_go_error()`, which wraps
//! it with `errors::from_value` (and its `Unwrap()` result, when it has
//! one), so `errors::as_type::<T>` finds it. `errors::as_type` returns a
//! copy: code that changes a found Go error in place builds a new one.

use crate::contentmapper::prelude::*;

use crate::flags_macros::go_enum;
use crate::locale::Locale;
use std::sync::Arc;
use std::time::Duration;

// Go: contentmapper/host.go:14 TransformErrorKind
// TransformErrorKind identifies the stage at which a content mapper transform failed.
go_enum!(TransformErrorKind, u8 {
    UNKNOWN = 0;
    INITIALIZE = 1;
    PROJECT = 2;
    REQUEST = 3;
    RESPONSE = 4;
    MAPPINGS = 5;
});

// Go: contentmapper/host.go:26 TransformError
// TransformError reports a failure while preparing, requesting, or decoding a transform.
#[derive(Clone, Debug, PartialEq)]
pub struct TransformError {
    pub kind: TransformErrorKind,
    err: Option<GoError>,
}

// Go: contentmapper/host.go:32 NewTransformError
// NewTransformError creates a transform error for the given stage and underlying error.
#[must_use]
pub fn new_transform_error(kind: TransformErrorKind, err: Option<GoError>) -> TransformError {
    TransformError { kind, err }
}

impl TransformError {
    // Go: contentmapper/host.go:36 TransformError.Error
    #[must_use]
    pub fn error(&self) -> String {
        // Go `%v` of a nil error prints `<nil>`.
        match &self.err {
            Some(err) => format!("content mapper transform failed: {}", err.error()),
            None => "content mapper transform failed: <nil>".to_string(),
        }
    }

    // Go: contentmapper/host.go:40 TransformError.Unwrap
    #[must_use]
    pub fn unwrap(&self) -> Option<GoError> {
        self.err.clone()
    }

}

impl std::fmt::Display for TransformError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.error())
    }
}

// Go: contentmapper/host.go:43 DiagnosticDirectiveErrorKind
// DiagnosticDirectiveErrorKind identifies why a diagnostic directive was rejected.
go_enum!(DiagnosticDirectiveErrorKind, u8 {
    INVALID_RANGE = 0;
    INVALID_POLICY = 1;
    EXPECT_MISSING_UNUSED_DIAGNOSTIC = 2;
    INVALID_UNUSED_DIAGNOSTIC_INDEX = 3;
    OVERLAP = 4;
});

// Go: contentmapper/host.go:54 DiagnosticDirectiveError
// DiagnosticDirectiveError reports an invalid diagnostic directive in a transform response.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DiagnosticDirectiveError {
    pub kind: DiagnosticDirectiveErrorKind,
    pub index: i32,
    pub supplemental_index: i32,
    pub policy: DiagnosticDirectivePolicy,
}

impl DiagnosticDirectiveError {
    // Go: contentmapper/host.go:61 DiagnosticDirectiveError.Error
    #[must_use]
    pub fn error(&self) -> String {
        format!("invalid content mapper diagnostic directive {}", self.index)
    }

}

impl std::fmt::Display for DiagnosticDirectiveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.error())
    }
}

// Go: contentmapper/host.go:66 InvalidVirtualExtensionError
// InvalidVirtualExtensionError reports an unsupported or missing virtual extension on a mapped output.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct InvalidVirtualExtensionError {
    pub extension: String,
}

impl InvalidVirtualExtensionError {
    // Go: contentmapper/host.go:70 InvalidVirtualExtensionError.Error

}


// Go: contentmapper/host.go:75 ProjectErrorKind
// ProjectErrorKind identifies why a mapper's openProject response was rejected.
go_enum!(ProjectErrorKind, u8 {
    MALFORMED_RESPONSE = 0;
    MISSING_CONFIG_IDENTITY = 1;
    NON_ABSOLUTE_WATCHED_FILE = 2;
    UNEXPECTED_CONFIG_IDENTITY = 3;
    UNEXPECTED_WATCHED_FILES = 4;
});

// Go: contentmapper/host.go:86 ProjectError
// ProjectError reports an invalid mapper openProject response.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProjectError {
    pub kind: ProjectErrorKind,
}

impl ProjectError {
    // Go: contentmapper/host.go:90 ProjectError.Error

}


// Go: contentmapper/host.go:108 InitializeErrorKind
// InitializeErrorKind identifies why a mapper's initialize response was rejected.
go_enum!(InitializeErrorKind, u8 {
    PROCESS_START = 0;
    PROCESS_EXIT = 1;
    NO_RESPONSE = 2;
    INVALID_RESPONSE = 3;
    REQUEST = 4;
    POSITION_ENCODING = 5;
    EMPTY_DIAGNOSTIC_SOURCE = 6;
    RESERVED_DIAGNOSTIC_SOURCE = 7;
});

// Go: contentmapper/host.go:122 InitializeError
// InitializeError reports an invalid or unsupported mapper initialize response.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct InitializeError {
    pub kind: InitializeErrorKind,
    pub mapper_name: String,
    pub command: String,
    pub detail: String,
    pub exit_code: i32,
    pub timeout_seconds: i32,
    pub position_encoding: PositionEncoding,
    pub diagnostic_source: String,
}

// Go: contentmapper/host.go:134 SupplementalFileCollisionError
// SupplementalFileCollisionError reports a compiler-assigned supplemental filename that already exists.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SupplementalFileCollisionError {
    pub file_name: String,
}

impl SupplementalFileCollisionError {
    // Go: contentmapper/host.go:138 SupplementalFileCollisionError.Error

}


impl InitializeError {
    // Go: contentmapper/host.go:142 InitializeError.Error

}


// Go: contentmapper/host.go:166 Result
// Result is the outcome of transforming a content-mapped source file into virtual TypeScript.
// PORT: the name shadows `std::result::Result` in this package; package
// files write `std::result::Result` in full. Go `*spanmap.SpanMap` is
// `Option<Arc<SpanMap>>` (the source files and checker threads share it).
// Go `*ast.Diagnostic` is `crate::core::Diagnostic`.

// Go: contentmapper/host.go:184 MappedResult
// MappedResult is one virtual source file and its mapping to the original input.
#[derive(Clone, Default)]
pub struct MappedResult {
    pub text: String,
    pub virtual_extension: String,
    pub mappings: Option<Arc<spanmap::SpanMap>>,
    pub diagnostic_directives: Vec<ast::MappedDiagnosticDirective>,
}

// Go: contentmapper/host.go:192 Request
// Request carries the inputs for transforming one content-mapped source file.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Request {
    // FileName is the content-mapped source file being transformed.
    pub file_name: String,
    // Content is the content-mapped source file's text.
    pub content: String,
}

// Go: contentmapper/host.go:200 ProjectSpec
// ProjectSpec describes the project configuration visible to its content mappers.
// PORT: Go `*core.CompilerOptions` is `Option<Rc<CompilerOptions>>` (nil is
// `None`); the host keys projects by its pointer, as Go does.

// Go: contentmapper/host.go:209 OptionPathSegment
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OptionPathSegment {
    pub property: String,
    pub index: i32,
    pub is_index: bool,
}

// Go: contentmapper/host.go:215 OptionDiagnostic
#[derive(Clone, Debug)]
pub struct OptionDiagnostic {
    pub mapper: Rc<Mapper>,
    pub path: Vec<OptionPathSegment>,
    pub source: String,
    pub code: i32,
    pub message_text: String,
}

// Go: contentmapper/host.go:224 OperationTiming
// OperationTiming is the cumulative wall time and invocation count for one mapper operation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OperationTiming {
    pub count: u64,
    pub duration: Duration,
}

// Go: contentmapper/host.go:230 MapperTimings
// MapperTimings is cumulative process and protocol activity for one resolved mapper identity.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MapperTimings {
    pub spawn: OperationTiming,
    pub initialize: OperationTiming,
    pub open_project: OperationTiming,
    pub close_project: OperationTiming,
    pub transform: OperationTiming,
}

// Go: contentmapper/host.go:239 Timings
// Timings is a cumulative snapshot of content mapper process and protocol activity.
// PORT: Go `map[string]MapperTimings` is an `IndexMap` (Go map order is
// random; readers sort or look up by identity).


// Go: contentmapper/host.go:263 operationTimingSince
fn operation_timing_since(current: OperationTiming, previous: OperationTiming) -> OperationTiming {
    OperationTiming {
        count: current.count - current.count.min(previous.count),
        duration: current.duration.saturating_sub(previous.duration),
    }
}

// Go: contentmapper/host.go:273 Project
// Project is the project-scoped view of a Host. It owns mapper configuration handles and provides the
// identities and watch dependencies needed for caching and incremental builds. Mapper projects are opened
// lazily when a transform is requested, or earlier when dynamic configuration is needed.

// Go: contentmapper/host.go:295 Host
// Host transforms otherwise unsupported file content into virtual TypeScript during program construction, by driving the
// configured content mappers. Create one with NewHost; Close tears down every mapper it spawned.
