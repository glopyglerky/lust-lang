//! Port of Go `transformers/estransforms/objectrestspread.go`.

use super::contract::{};
use crate::prelude::*;
use crate::printer::{EmitFlags};
use crate::transformers::destructuring::{
    FlattenLevel, 
};

// Go: transformers/estransforms/objectrestspread.go:10 objectRestSpreadTransformer


/// Go `oldParamScope`.
type OldParamScope = Option<FxHashSet<Node>>;


// Go: transformers/estransforms/objectrestspread.go:590 newObjectRestSpreadTransformer
