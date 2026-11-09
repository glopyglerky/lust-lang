//! Port of Go `transformers/estransforms/esdecorator.go` lines 1722 to 2718:
//! `visitThisExpression` through `createSymbolMetadataReference`. The first
//! part is in `es_decorator.rs`.

use super::es_decorator::{
    ClassInfoRef, 
    
};
use super::named_evaluation::{};
use crate::prelude::*;
use crate::printer::{AutoGenerateOptions, EmitFlags};
use crate::transformers::utilities::{
    get_non_assignment_operator_for_compound_assignment, 
    
};

