// Go: internal/rules/unnecessary_pipe.go

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;



/// UnnecessaryPipeMatch holds the diagnostic and parsed pipe call result needed
/// by both the diagnostic rule and the quick-fix.
#[derive(Clone, Debug)]
pub struct UnnecessaryPipeMatch {
    /// The source file where this match was found
    pub source_file: Node,
    /// The pre-computed error range for this match
    pub location: TextRange,
    /// The parsed pipe call (contains Subject node and call Node)
    pub result: Rc<ParsedPipeCallResult>,
}

// Go: rules/unnecessary_pipe.go AnalyzeUnnecessaryPipe
