//! Port of `transformers/declarations/util.go`.
//!
//! These helpers are used by the declaration transformer body. Most of that
//! body is not ported yet, so some are not called yet.
#![allow(dead_code)]

use crate::prelude::*;
use crate::printer::{};

// Go: transformers/declarations/util.go:9 needsScopeMarker

// Go: transformers/declarations/util.go:13 canHaveLiteralInitializer

// Go: transformers/declarations/util.go:25 canProduceDiagnostics
// PERF: Go chains 23 `ast.Is*` tests. Each one is a pure kind compare, so one
// kind read and one match give the same result. The arms keep the Go order.

/// `can_produce_diagnostics` of a node of kind `kind`.
pub(crate) fn can_produce_diagnostics_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::VariableDeclaration
            | SyntaxKind::PropertyDeclaration
            | SyntaxKind::PropertySignature
            | SyntaxKind::BindingElement
            | SyntaxKind::SetAccessor
            | SyntaxKind::GetAccessor
            | SyntaxKind::ConstructSignature
            | SyntaxKind::CallSignature
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::MethodSignature
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::Parameter
            | SyntaxKind::TypeParameter
            | SyntaxKind::ExpressionWithTypeArguments
            | SyntaxKind::ImportEqualsDeclaration
            | SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::JsTypeAliasDeclaration
            | SyntaxKind::Constructor
            | SyntaxKind::IndexSignature
            | SyntaxKind::PropertyAccessExpression
            | SyntaxKind::ElementAccessExpression
            | SyntaxKind::BinaryExpression
            | SyntaxKind::CallExpression // !!! TODO: JSDoc support (ast.IsJSDocTypeAlias)
    )
}

// Go: transformers/declarations/util.go:52 canReuseModifierNodes

// Go: transformers/declarations/util.go:61 isDeclarationAndNotVisible

// Go: transformers/declarations/util.go:87 getBindingNameVisible

// Go: transformers/declarations/util.go:109 isEnclosingDeclaration


// Go: transformers/declarations/util.go:122 isAlwaysType

// Go: transformers/declarations/util.go:129 maskModifierFlags

// Go: transformers/declarations/util.go:142 unwrapParenthesizedExpression

// Go: transformers/declarations/util.go:149 isPrivateMethodTypeParameter

// Go: transformers/declarations/util.go:155 shouldEmitFunctionProperties
// Returns true if expando properties should be emitted for this function.
// Properties are emitted if any overload in the symbol has a body (implementation).
// PORT: `input.Symbol.Declarations` is the binder symbol, read from the
// program's binder symbols (`program::bound_symbols`).

// Go: transformers/declarations/util.go:164 getEffectiveBaseTypeNode

// Go: transformers/declarations/util.go:177 isScopeMarker

// Go: transformers/declarations/util.go:181 hasScopeMarker
// PORT: Go `*ast.StatementList` (nil allowed) is `Option<&[Node]>`.
