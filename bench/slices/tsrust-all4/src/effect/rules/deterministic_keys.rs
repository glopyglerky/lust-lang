// Go: internal/rules/deterministic_keys.go

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::keybuilder;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;



/// DeterministicKeyMatch holds the AST nodes and computed key info needed by both
/// the diagnostic rule and the quick-fix for the deterministicKeys pattern.
#[derive(Clone, Debug)]
pub struct DeterministicKeyMatch {
    /// The source file of the match
    pub source_file: Node,
    /// The pre-computed error range for the key string literal
    pub location: TextRange,
    /// The key string literal node
    pub key_string_literal: Node,
    /// The actual key string found in the source
    pub actual_key: String,
    /// The expected key computed by keybuilder
    pub expected_key: String,
}

// Go: rules/deterministic_keys.go AnalyzeDeterministicKeys

/// deterministicKeyMatch holds the matched key info from a class declaration.
// PORT: Go `deterministicKeyMatch`. Its Rust name would collide with the
// exported `DeterministicKeyMatch`, so it has an `Inner` suffix.
#[derive(Clone, Debug)]
pub struct DeterministicKeyMatchInner {
    pub class_name: Node,
    pub key_string_literal: Node,
    pub target: String,
}

// Go: rules/deterministic_keys.go checkDeterministicKeyMatch

// Go: rules/deterministic_keys.go matchClassPattern

// Go: rules/deterministic_keys.go matchCustomPattern

// Go: rules/deterministic_keys.go getPackageJsonDirectory
