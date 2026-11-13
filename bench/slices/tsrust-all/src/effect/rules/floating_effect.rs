//! Port of Effect-TS/tsgo `internal/rules/floating_effect.go`.
//! Package rules contains all Effect diagnostic rule implementations.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;

/// floatingEffectResult holds information about a detected floating Effect expression.
pub struct FloatingEffectResult {
    /// isStrict is true when the type's symbol name is exactly "Effect"
    pub is_strict: bool,
    /// exprType is the checker type of the floating expression
    pub expr_type: TypeId,
}



/// FloatingEffectMatch holds the diagnostic location and expression needed by
/// both the diagnostic rule and its quick-fix.
pub struct FloatingEffectMatch {
    pub source_file: Node,
    pub location: TextRange,
    pub expression: Node,
    pub is_strict: bool,
    pub expr_type: TypeId,
}



