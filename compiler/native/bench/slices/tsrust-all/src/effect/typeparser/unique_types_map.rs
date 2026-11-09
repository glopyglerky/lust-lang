// Go: internal/typeparser/unique_types_map.go

use crate::effect::typeparser::*;
use crate::prelude::*;

/// UniqueTypesResult holds the result of AppendToUniqueTypesMap.
#[derive(Clone, Debug, Default)]
pub struct UniqueTypesResult {
    /// All type indexes encountered (both new and existing)
    pub all_indexes: Vec<String>,
}

