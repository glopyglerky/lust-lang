//! Go `core` and `ast` pieces that only the frontend needs.

use crate::frontend::prelude::*;

// Go: core/typeacquisition.go:5 TypeAcquisition
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TypeAcquisition {
    pub enable: Tristate,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub disable_filename_based_type_acquisition: Tristate,
}

// Go: core/projectreference.go:5 ProjectReference
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProjectReference {
    // Path is a normalized path on disk.
    pub path: String,
    // OriginalPath is the path as it was originally written.
    pub original_path: String,
    // Circular indicates that this reference is intended to form a circularity.
    pub circular: bool,
}

// Go: core/projectreference.go:5 ProjectReference (JSON v2 struct marshaler)
// PORT: Go marshals the struct by reflection in field order with the tags
// `json:"path"`, `json:"originalPath,omitempty"` and
// `json:"circular,omitempty"` (tsgo#4627, ts#64326). v2 `omitempty` drops an
// empty string but writes `false`.

// Go: core/projectreference.go:14 ResolveProjectReferencePath

// Go: core/projectreference.go:18 ResolveConfigFileNameOfProjectReference

/// Go interface `ast.HasFileName`.
pub trait HasFileName {
    fn file_name(&self) -> String;
    fn path(&self) -> Path;
}

impl HasFileName for HasFileNameImpl {
    fn file_name(&self) -> String {
        self.file_name.clone()
    }
    fn path(&self) -> Path {
        Path(self.path.clone())
    }
}

// Go: ast.SourceFile implements `ast.HasFileName`.

// Go: core/core.go:525 GetScriptKindFromFileName

// Go: core/core.go:544 GetDefaultExtensionForScriptKind (tsgo#4712)

// EnsureScriptKindFromFileName is like GetScriptKindFromFileName, but defaults to
// ScriptKindTS when the file name has no recognized extension (e.g. files included
// with allowNonTsExtensions), so the result is always safe to hand to the parser.
// Go: core/core.go:562 EnsureScriptKindFromFileName
