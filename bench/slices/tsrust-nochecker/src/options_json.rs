//! The Go JSON v2 form of `core.CompilerOptions` and `core.TypeAcquisition`
//! (the `json` tags in core/compileroptions.go and core/typeacquisition.go),
//! and the v2 `omitempty` member rule.
//!
//! PORT: Go marshals these structs by reflection. The port wrote them by
//! hand for `api/proto.go`. Bump B wave 3 (tsgo#4712) moved them here from
//! `api/proto.rs`, because the content mappers (`crate::contentmapper`, on
//! the compiler side) use them too and must not depend on `api`.
//! `api::proto` re-exports them.

use crate::prelude::*;

use crate::frontend::json::{
    JsonDecoder, JsonError, MarshalerTo, UnmarshalerFrom, 
};
use crate::frontend::json_ext::{
    SemanticError, 
    write_object_end, write_object_start,
};


// ---------------------------------------------------------------------------
// core.CompilerOptions JSON
// ---------------------------------------------------------------------------


// A `Tristate` member with `omitzero` (`TSUnknown` is the zero value).

// Go JSON v2 struct marshaler for `core.TypeAcquisition` (tags `enable`,
// `include`, `exclude` and `disableFilenameBasedTypeAcquisition`, all
// `omitzero`).
// PORT: a Go nil `Include` or `Exclude` is omitted and a non-nil empty one
// is written as `[]`. The Rust `Vec` has no nil, so an empty list is
// omitted.
pub struct TypeAcquisitionJSON<'a>(pub &'a crate::frontend::core_ext::TypeAcquisition);


// Go `collections.OrderedMap[string, []string]` MarshalJSONTo.
struct PathsJSON<'a>(&'a IndexMap<String, Option<Vec<String>>>);


// Go: core/compileroptions.go:14 PluginImport (ts#64397), tag `json:"name"`.
// PORT: Go marshals and unmarshals it by reflection (the v2 default struct
// arshalers).




// Go v2 calls the legacy `Tristate.UnmarshalJSON` with the raw value, also
// for `null` (arshal_methods.go:286). It never fails.

// Go v2 int arshaler for a Go named `int32` type (`core.ModuleKind`): the
// `int32` decode, with errors that name `go_type`.

// Go v2 pointer arshaler for a `*int` member: `null` sets nil. Go `int` is
// 64-bit; errors name Go `int`.
