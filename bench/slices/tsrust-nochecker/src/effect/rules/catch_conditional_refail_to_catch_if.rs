// Go: internal/rules/catch_conditional_refail_to_catch_if.go

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;



#[derive(Clone, Debug)]
pub struct CatchConditionalRefailToCatchIfMatch {
    pub source_file: Node,
    pub location: TextRange,
    pub catch_method_name: String,
    pub fail_method_name: String,
    pub preferred_method_name: String,
}

/// Go `conditionalRefailCatchMethods`.
#[derive(Clone, Copy, Debug, Default)]
pub struct ConditionalRefailCatchMethods {
    pub catch_method_name: &'static str,
    pub fail_method_name: &'static str,
    pub preferred_method_name: &'static str,
}

// Go: rules/catch_conditional_refail_to_catch_if.go AnalyzeCatchConditionalRefailToCatchIf

// Go: rules/catch_conditional_refail_to_catch_if.go conditionalRefailMethods

// Go: rules/catch_conditional_refail_to_catch_if.go analyzeConditionalRefailHandler

// Go: rules/catch_conditional_refail_to_catch_if.go isConditionalRefailExpression

// Go: rules/catch_conditional_refail_to_catch_if.go isConditionalRefailPredicate

// Go: rules/catch_conditional_refail_to_catch_if.go conditionalRefailNodeContainsParameter
