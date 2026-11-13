//! Port of Go `transformers/moduletransforms/commonjsmodule.go` lines 978 to
//! 2160: `visitTopLevelVariableStatement` through `getExports`.

use super::commonjs_module::{VisitorKind};
use super::utilities::{
    
    
    
};
use crate::prelude::*;
use crate::transformers::destructuring::{FlattenLevel, };
use crate::transformers::utilities::{
    
    
};

