//! Port of Go `internal/transformers/declarations`: the declaration emit
//! transformer and the diagnostics it reports.

pub mod diagnostics;
pub mod supplementalreferences;
pub mod tracker;
pub mod transform;
pub mod transform_p2;
pub mod transform_p3;
pub mod transform_p4;
pub mod util;

pub use diagnostics::*;
pub use supplementalreferences::*;
pub use tracker::*;
// The crate prelude also exports the checker `SymbolTrackerImpl`.
pub use transform::*;

use crate::prelude::*;

// Go: transformers/declarations/transform.go:28 OutputPaths
pub trait OutputPaths {
    fn declaration_file_path(&self) -> String;
    fn js_file_path(&self) -> String;
}

// Go: transformers/declarations/transform.go:34 DeclarationEmitHost
// Used to be passed in the TransformationContext, which is now just an EmitContext
// PORT: Go embeds `modulespecifiers.ModuleSpecifierGenerationHost`. That
// interface is not ported, so its methods are not part of this trait yet.
