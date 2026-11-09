//! Port of Effect-TS/tsgo `internal/rules/outdated_api.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;

use super::outdated_api_db::{Migration, MigrationTag};


// Go: OutdatedApi.Run

/// Go `OutdatedApiMatch`: holds the analysis results for one outdated API
/// usage.
#[derive(Clone, Debug)]
pub struct OutdatedApiMatch {
    pub source_file: Node,
    pub location: TextRange,
    pub property_name: String,
    pub migration_hint: String,
    pub migration: Migration,
}

// Go: AnalyzeOutdatedApi

// Go: checkOutdatedApiPropertyAccess

// Go: migrationHintText
fn migration_hint_text(m: &Migration) -> String {
    match m.tag {
        MigrationTag::Removed => m.alternative_pattern.clone(),
        MigrationTag::RenamedSameBehaviour => format!("Renamed to {}.", m.new_name),
        MigrationTag::RenamedAndNeedsOptions => {
            format!("Renamed to {}. {}", m.new_name, m.options_instructions)
        }
        _ => String::new(),
    }
}
