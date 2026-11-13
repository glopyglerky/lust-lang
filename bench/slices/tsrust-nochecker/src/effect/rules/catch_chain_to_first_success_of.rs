//! Port of Effect-TS/tsgo `internal/rules/catch_chain_to_first_success_of.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;



// Go: rules/catch_chain_to_first_success_of.go CatchChainToFirstSuccessOfMatch
#[derive(Clone, Copy)]
pub struct CatchChainToFirstSuccessOfMatch {
    pub source_file: Node,
    pub location: TextRange,
}

// Go: rules/catch_chain_to_first_success_of.go catchChainToFirstSuccessOfCandidate
// PORT: Go keeps a pointer into the flow's transformations; the port keeps a copy.


// Go: rules/catch_chain_to_first_success_of.go analyzeCatchChainToFirstSuccessOfCandidate
// PORT: Go returns a zero candidate with `false`; the port returns `None` there.

// Go: rules/catch_chain_to_first_success_of.go catchChainPreservesFirstSuccessOfError
