use crate::ls::prelude::*;

// Go `internal/ls/inlay_hints.go`: textDocument/inlayHint.

use crate::frontend::stringutil_ls;
use crate::spanmap::Feature;


// Go: ls/inlay_hints.go:59 inlayHintState
// PORT: Go `checker *checker.Checker` is the leased checker, borrowed for the
// request (`&mut Checker`).

// Go: ls/inlay_hints.go:828 parameterInfo
struct ParameterInfo {
    parameter: Node,
    name: String,
    is_rest_parameter: bool,
}



// Go: ls/inlay_hints.go:396 shouldShowParameterNameHints
fn should_show_parameter_name_hints(preferences: &lsutil::InlayHintsPreferences) -> bool {
    preferences.include_inlay_parameter_name_hints
        == lsutil::IncludeInlayParameterNameHints::LITERALS
        || preferences.include_inlay_parameter_name_hints
            == lsutil::IncludeInlayParameterNameHints::ALL
}

// Go: ls/inlay_hints.go:401 shouldShowLiteralParameterNameHintsOnly
fn should_show_literal_parameter_name_hints_only(
    preferences: &lsutil::InlayHintsPreferences,
) -> bool {
    preferences.include_inlay_parameter_name_hints
        == lsutil::IncludeInlayParameterNameHints::LITERALS
}

// Go: ls/inlay_hints.go:406 isSignatureSupportingReturnAnnotation
// node is FunctionDeclaration | ArrowFunction | FunctionExpression | MethodDeclaration | GetAccessor

// Go: ls/inlay_hints.go:411 isHintableDeclaration

// Go: ls/inlay_hints.go:421 isHintableLiteral
// PORT: Go calls `ast.IsInfinityOrNaNString`, which has no ast port; the
// checker port `is_infinity_or_nan_string` has the same body.

// Go: ls/inlay_hints.go:436 isModuleReferenceType
// PORT: types and symbols live in the checker arenas, so the checker is a parameter.

// Go: ls/inlay_hints.go:899 getParameterDeclarationIdentifier
// PORT: symbols live in the checker arena, so it is a parameter (PORTING
// "Names": functions that take a symbol get `symbols: &SymbolArena`).

// Go: ls/inlay_hints.go:906 identifierOrAccessExpressionPostfixMatchesParameterName

// Go: ls/inlay_hints.go:943 isAnyInlayHintEnabled
fn is_any_inlay_hint_enabled(preferences: &lsutil::InlayHintsPreferences) -> bool {
    preferences.include_inlay_parameter_name_hints != lsutil::IncludeInlayParameterNameHints::NONE
        || preferences
            .include_inlay_function_parameter_type_hints
            .is_true()
        || preferences.include_inlay_variable_type_hints.is_true()
        || preferences
            .include_inlay_property_declaration_type_hints
            .is_true()
        || preferences
            .include_inlay_function_like_return_type_hints
            .is_true()
        || preferences.include_inlay_enum_member_value_hints.is_true()
}
