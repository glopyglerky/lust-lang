//! Go: module/staticresolver.go (ts#64299).

use crate::frontend::prelude::*;
use crate::gostd::{GoError, errors, strconv};
use std::sync::Arc;

// Go: module/staticresolver.go:12 StaticResolutionEntry
// PORT: Go `*core.ResolutionMode` and `*ResolvedModule` are `Option`.

// Go: module/staticresolver.go:19 staticResolutionKey
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
struct StaticResolutionKey {
    module_name: String,
    directory: Path,
    mode: ResolutionMode,
    has_directory: bool,
    has_mode: bool,
}

// Go: module/staticresolver.go:27 StaticResolutions

// Go: module/staticresolver.go:34 NewStaticResolutions


// Go: module/staticresolver.go:83 StaticResolver

// Go: module/staticresolver.go:88 NewStaticResolver


