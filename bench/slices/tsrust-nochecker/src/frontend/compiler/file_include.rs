//! Port of compiler/fileInclude.go: why a file is part of the program, and
//! the "explain files" diagnostics for each reason.

use crate::frontend::prelude::*;
use std::cell::OnceCell;

// Go: fileInclude.go:15 fileIncludeKind
// PORT: Go `int` enum with iota constants. The newtype keeps the Go order,
// so `is_referenced_file` can compare with `<=`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileIncludeKind(pub i32);

impl FileIncludeKind {
    // References from file
    pub const IMPORT: FileIncludeKind = FileIncludeKind(0);
    pub const REFERENCE_FILE: FileIncludeKind = FileIncludeKind(1);
    pub const TYPE_REFERENCE_DIRECTIVE: FileIncludeKind = FileIncludeKind(2);
    pub const LIB_REFERENCE_DIRECTIVE: FileIncludeKind = FileIncludeKind(3);

    pub const ROOT_FILE: FileIncludeKind = FileIncludeKind(4);
    pub const LIB_FILE: FileIncludeKind = FileIncludeKind(5);
    pub const AUTOMATIC_TYPE_DIRECTIVE_FILE: FileIncludeKind = FileIncludeKind(6);
    // tsgo#4712
    pub const CONTENT_MAPPER_SUPPLEMENTAL: FileIncludeKind = FileIncludeKind(7);
}

/// Go `FileIncludeReason.data any`.
// PORT: Go stores an `int`, a `*referencedFileData`, a
// `*automaticTypeDirectiveFileData`, a `tspath.Path` (the canonical file of
// a content mapper supplemental file, tsgo#4712) or nil in an `any` field.
// `None` is the Go nil (a default lib file reason has no index).
#[derive(Clone, Debug, Default)]
pub enum FileIncludeData {
    #[default]
    None,
    Index(i32),
    ReferencedFile(ReferencedFileData),
    AutomaticTypeDirectiveFile(AutomaticTypeDirectiveFileData),
    Path(Path),
}

// Go: fileInclude.go:29 FileIncludeReason
// PORT: `sync.Once` + pointer pairs are `OnceCell<Diagnostic>`.

// PORT: Go builds reasons with `&FileIncludeReason{kind: .., data: ..}`.
// This is that struct literal; the reason is shared by pointer in Go.

// Go: fileInclude.go:41 referencedFileData
// PORT: `synthetic` is `Node::NIL` for the Go nil.
#[derive(Clone, Debug, Default)]
pub struct ReferencedFileData {
    pub file: Path,
    pub index: i32,
    pub synthetic: Node,
}

// Go: fileInclude.go:47 referenceFileLocation
// PORT: `node` is `Node::NIL` for the Go nil. `ref_` is `None` for the Go nil.


// Go: fileInclude.go:75 automaticTypeDirectiveFileData
#[derive(Clone, Debug, Default)]
pub struct AutomaticTypeDirectiveFileData {
    pub type_reference: String,
    pub package_id: PackageId,
}

