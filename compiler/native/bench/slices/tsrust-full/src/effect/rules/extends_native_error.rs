//! Port of Effect-TS/tsgo `internal/rules/extends_native_error.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;

// Go: rules/extends_native_error.go ExtendsNativeError


// Go: rules/extends_native_error.go ExtendsNativeErrorMatch
#[derive(Clone, Copy)]
pub struct ExtendsNativeErrorMatch {
    pub source_file: Node,
    pub location: TextRange,
}

// Go: rules/extends_native_error.go AnalyzeExtendsNativeError

// Go: rules/extends_native_error.go checkExtendsNativeError
