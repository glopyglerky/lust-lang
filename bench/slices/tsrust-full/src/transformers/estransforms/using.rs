//! Port of Go `transformers/estransforms/using.go`.

use super::contract::{};
use super::named_evaluation::{};
use crate::prelude::*;
use crate::printer::{AutoGenerateOptions, EmitFlags, GeneratedIdentifierFlags};
use crate::transformers::utilities::{
    
};

// Go: transformers/estransforms/using.go:11 usingDeclarationTransformer


// Go: transformers/estransforms/using.go:21 newUsingDeclarationTransformer

// Go: transformers/estransforms/using.go:26 usingKind
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum UsingKind {
    None,
    Sync,
    Async,
}

/// Go `printer.AutoGenerateOptions{Flags: ReservedInNestedScopes | FileLevel | Optimistic}`.
fn default_export_binding_options() -> AutoGenerateOptions {
    AutoGenerateOptions {
        flags: GeneratedIdentifierFlags::RESERVED_IN_NESTED_SCOPES
            | GeneratedIdentifierFlags::FILE_LEVEL
            | GeneratedIdentifierFlags::OPTIMISTIC,
        ..Default::default()
    }
}


// Go: transformers/estransforms/using.go:761 isUsingVariableDeclarationList

// Go: transformers/estransforms/using.go:765 getUsingKindOfVariableDeclarationList

// Go: transformers/estransforms/using.go:776 getUsingKindOfVariableStatement

// Go: transformers/estransforms/using.go:780 getUsingKind

// Go: transformers/estransforms/using.go:787 getUsingKindOfStatements
