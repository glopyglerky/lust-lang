//! Port of Effect-TS/tsgo `internal/rules/unnecessary_pipe_chain.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;



/// UnnecessaryPipeChainMatch holds the diagnostic and parsed pipe call results
/// needed by both the diagnostic rule and the quick-fix.
#[derive(Clone, Debug)]
pub struct UnnecessaryPipeChainMatch {
    /// The source file where the diagnostic should be reported
    pub source_file: Node,
    /// The pre-computed error range for this match
    pub location: TextRange,
    /// The outer pipe call parse result
    pub outer: Rc<ParsedPipeCallResult>,
    /// The inner pipe call parse result (subject of outer)
    pub inner: Rc<ParsedPipeCallResult>,
}



