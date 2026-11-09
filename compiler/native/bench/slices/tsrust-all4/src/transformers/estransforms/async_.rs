//! Port of Go `transformers/estransforms/async.go`.

use super::contract::{};
use crate::prelude::*;
use crate::printer::{
    AutoGenerateOptions, EmitFlags,
    GeneratedIdentifierFlags,
};

// Go: transformers/estransforms/async.go:12 asyncContextFlags
type AsyncContextFlags = i32;

const ASYNC_CONTEXT_NON_TOP_LEVEL: AsyncContextFlags = 1 << 0;
const ASYNC_CONTEXT_HAS_LEXICAL_THIS: AsyncContextFlags = 1 << 1;

// Go: transformers/estransforms/async.go:19 lexicalArgumentsInfo
#[derive(Clone, Copy)]
struct LexicalArgumentsInfo {
    binding: Node,
    used: bool,
}

impl Default for LexicalArgumentsInfo {
    fn default() -> Self {
        Self {
            binding: Node::NIL,
            used: false,
        }
    }
}

// Go: transformers/estransforms/async.go:24 asyncTransformer


// Go: transformers/estransforms/async.go:37 newAsyncTransformer

/// Go `printer.AutoGenerateOptions{Flags: Optimistic | FileLevel}`.
fn optimistic_file_level() -> AutoGenerateOptions {
    AutoGenerateOptions {
        flags: GeneratedIdentifierFlags::OPTIMISTIC | GeneratedIdentifierFlags::FILE_LEVEL,
        ..Default::default()
    }
}


// Go: transformers/estransforms/async.go:952 isSimpleParameterList

// Go: transformers/estransforms/async.go:963 isNodeWithPossibleHoistedDeclaration

// Go: transformers/estransforms/async.go:899 assignmentTargetContainsSuperProperty

// Go: transformers/estransforms/async.go:931 isUpdateExpression
