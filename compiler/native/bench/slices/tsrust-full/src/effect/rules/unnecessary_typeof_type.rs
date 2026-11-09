// Go: internal/rules/unnecessary_typeof_type.go

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;



#[derive(Clone, Debug)]
pub struct UnnecessaryTypeofTypeMatch {
    pub source_file: Node,
    pub location: TextRange,
    pub query_node: Node,
    pub inner_entity_name: Node,
    pub replacement_text: String,
}

// Go: rules/unnecessary_typeof_type.go AnalyzeUnnecessaryTypeofType

// Go: rules/unnecessary_typeof_type.go analyzeUnnecessaryTypeofTypeNode

// Go: rules/unnecessary_typeof_type.go resolveEntityNameAsType

// Go: rules/unnecessary_typeof_type.go resolveQualifiedTypeMember

// Go: rules/unnecessary_typeof_type.go resolvedSymbolType

// Go: rules/unnecessary_typeof_type.go resolveAliasSymbol

// Go: rules/unnecessary_typeof_type.go isSelfReferentialTypeAliasReference
// PORT: takes the checker to read `symbol.Declarations`.
