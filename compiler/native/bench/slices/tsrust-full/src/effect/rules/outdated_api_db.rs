//! Port of Effect-TS/tsgo `internal/rules/outdated_api_db.go`: the Effect v3
//! to v4 migration table that the `outdatedApi` rule reads.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;
use std::sync::LazyLock;

/// Go `MigrationTag`: identifies the type of migration for an API. The
/// variants are the Go consts `MigrationTagUnchanged` and so on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MigrationTag {
    Unchanged,
    RenamedSameBehaviour,
    RenamedAndNeedsOptions,
    Removed,
}

impl MigrationTag {
    /// The Go string value.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            MigrationTag::Unchanged => "Unchanged",
            MigrationTag::RenamedSameBehaviour => "RenamedSameBehaviour",
            MigrationTag::RenamedAndNeedsOptions => "RenamedAndNeedsOptions",
            MigrationTag::Removed => "Removed",
        }
    }
}

/// Go `Migration`: describes how an Effect v3 API maps to v4.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Migration {
    pub tag: MigrationTag,
    /// for RenamedSameBehaviour and RenamedAndNeedsOptions
    pub new_name: String,
    /// for RenamedAndNeedsOptions
    pub options_instructions: String,
    /// for Removed
    pub alternative_pattern: String,
}

// Go: unchanged
pub fn unchanged() -> Migration {
    Migration {
        tag: MigrationTag::Unchanged,
        new_name: String::new(),
        options_instructions: String::new(),
        alternative_pattern: String::new(),
    }
}

// Go: renamedSameBehaviour
pub fn renamed_same_behaviour(new_name: &str) -> Migration {
    Migration {
        tag: MigrationTag::RenamedSameBehaviour,
        new_name: new_name.to_string(),
        options_instructions: String::new(),
        alternative_pattern: String::new(),
    }
}

// Go: renamedAndNeedsOptions
pub fn renamed_and_needs_options(new_name: &str, options_instructions: &str) -> Migration {
    Migration {
        tag: MigrationTag::RenamedAndNeedsOptions,
        new_name: new_name.to_string(),
        options_instructions: options_instructions.to_string(),
        alternative_pattern: String::new(),
    }
}

// Go: removed
pub fn removed(alternative_pattern: &str) -> Migration {
    Migration {
        tag: MigrationTag::Removed,
        new_name: String::new(),
        options_instructions: String::new(),
        alternative_pattern: alternative_pattern.to_string(),
    }
}

