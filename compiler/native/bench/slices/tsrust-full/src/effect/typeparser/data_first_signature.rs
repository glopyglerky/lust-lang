//! Port of Effect-TS/tsgo `internal/typeparser/data_first_signature.go`.

use crate::effect::typeparser::*;
use crate::prelude::*;

/// Go `ParsedDataFirstOrLastCall`.
#[derive(Clone, Debug)]
pub struct ParsedDataFirstOrLastCall {
    pub node: Node,
    pub callee: Node,
    pub subject: Node,
    pub args: Vec<Node>,
    pub subject_index: i32,
}



// Go: typeparser/data_first_signature.go MatchesPipeableSignature

// Go: typeparser/data_first_signature.go argumentTypesMatchParameters

// Go: typeparser/data_first_signature.go candidateAcceptsSubject

// Go: typeparser/data_first_signature.go hasCompatibleCallShape

// Go: typeparser/data_first_signature.go typeAcceptsArgument

// Go: typeparser/data_first_signature.go sameShallowTypeOrigin

// Go: typeparser/data_first_signature.go sameSymbolReference

// Go: typeparser/data_first_signature.go DerivePipeableSignatureFromDataFirst

// Go: typeparser/data_first_signature.go isLikelySelfParameter
// PORT: Go reads `sym.Name` without a checker; the symbol arena is on the
// checker here. Go `strings.ToLower` maps each rune with `unicode.ToLower`.

// Go: typeparser/data_first_signature.go omitArgAt
pub fn omit_arg_at(nodes: &[Node], index: usize) -> Vec<Node> {
    let mut result = Vec::with_capacity(nodes.len().saturating_sub(1));
    for (i, &node) in nodes.iter().enumerate() {
        if i == index {
            continue;
        }
        result.push(node);
    }
    result
}
