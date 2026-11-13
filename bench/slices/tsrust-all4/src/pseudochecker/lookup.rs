//! Port of Go `pseudochecker/lookup.go`.

use crate::prelude::*;

use super::PseudoChecker;
use super::types::*;

// PORT: every method that can reach a `node.Symbol.Declarations` read takes
// `symbols: &SymbolArena` (the checker's arena) as its first parameter. See
// the note on `PseudoChecker` in mod.rs.
// PORT: Go `debug.FailBadSyntaxKind` panics; so does the port
// (`gostd::debug::fail_bad_syntax_kind`). Go returns
// `nil` after it, which is unreachable, so the Rust return type is
// `Rc<PseudoType>` without `Option`, and Go `expr != nil` checks are always
// true.
impl PseudoChecker {
    // Go: pseudochecker/lookup.go:11 GetReturnTypeOfSignature

    // Go: pseudochecker/lookup.go:26 GetTypeOfAccessor

    // Go: pseudochecker/lookup.go:30 GetTypeOfExpression

    // Go: pseudochecker/lookup.go:34 GetTypeOfDeclaration

    // Go: pseudochecker/lookup.go:69 typeFromPropertyAssignment

    // Go: pseudochecker/lookup.go:88 typeFromExpandoProperty

    // Go: pseudochecker/lookup.go:98 typeFromProperty

    // Go: pseudochecker/lookup.go:124 typeFromVariable

    // Go: pseudochecker/lookup.go:146 typeFromAccessor

    // Go: pseudochecker/lookup.go:166 getTypeAnnotationFromAllAccessorDeclarations

    // Go: pseudochecker/lookup.go:177 getTypeAnnotationFromAccessor

    // Go: pseudochecker/lookup.go:201 createReturnFromSignature

    // Go: pseudochecker/lookup.go:216 typeFromSingleReturnExpression

    // Go: pseudochecker/lookup.go:262 typeFromExpression

    // Go: pseudochecker/lookup.go:315 typeFromObjectLiteral

    // Go: pseudochecker/lookup.go:363 getAccessorMember

    // Go: pseudochecker/lookup.go:406 canGetTypeFromObjectLiteral

    // Go: pseudochecker/lookup.go:438 typeFromArrayLiteral

    // Go: pseudochecker/lookup.go:457 canGetTypeFromArrayLiteral

    // Go: pseudochecker/lookup.go:494 typeFromPrimitiveLiteralPrefix

    // Go: pseudochecker/lookup.go:510 typeFromTypeAssertion

    // Go: pseudochecker/lookup.go:517 typeFromFunctionLikeExpression

    // Go: pseudochecker/lookup.go:532 cloneTypeParameters
    // PORT: Go returns `[]*ast.TypeParameterDeclaration`; nil is an empty Vec.

    // Go: pseudochecker/lookup.go:631 typeFromParameter

    // Go: pseudochecker/lookup.go:649 typeFromParameterWorker

    // Go: pseudochecker/lookup.go:687 cloneParameters
    // PORT: Go nil is an empty Vec.
}

// Go: pseudochecker/lookup.go:196 isValueSignatureDeclaration

// Go: pseudochecker/lookup.go:470 isConstContextPropagatingKind
/// See `isConstContext` in `checker.go` - this is basically any node kind mentioned in that
fn is_const_context_propagating_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::ArrayLiteralExpression
            | SyntaxKind::ObjectLiteralExpression
            | SyntaxKind::ParenthesizedExpression
            | SyntaxKind::SpreadElement
            | SyntaxKind::PropertyAssignment
            | SyntaxKind::ShorthandPropertyAssignment
            | SyntaxKind::TemplateSpan
            | SyntaxKind::PrefixUnaryExpression
    )
}

// Go: pseudochecker/lookup.go:482 IsInConstContext

// Go: pseudochecker/lookup.go:546 isUndefinedPseudoType
fn is_undefined_pseudo_type(t: &PseudoType) -> bool {
    t.kind == PseudoTypeKind::UNDEFINED
        || (t.kind == PseudoTypeKind::MAYBE_CONST_LOCATION
            && is_undefined_pseudo_type(&t.as_pseudo_type_maybe_const_location().const_type))
}

// Go: pseudochecker/lookup.go:550 typeNodeCouldReferToUndefined

// Go: pseudochecker/lookup.go:578 CouldAlreadyReferToUndefinedType

// Go: pseudochecker/lookup.go:597 isOptionalInitializedOrRestParameter

// Go: pseudochecker/lookup.go:610 lastRequiredParamIndex

// Go: pseudochecker/lookup.go:619 addUndefinedIfDefinitelyRequired

// Go: pseudochecker/lookup.go:715 isContextuallyTyped

#[cfg(test)]
mod debug_site_tests {
    use super::*;
    use crate::core::go_panic_text;
    use crate::frontend::parser::{SourceFileParseOptions, parse_source_file};
    use crate::frontend::tspath::Path;
    use crate::ls::findallreferences_p2::for_each_descendant_of_kind;
    use crate::pseudochecker::new_pseudo_checker;

    /// The last node of `kind` in a parse of `text`.
    fn node_of_kind(text: &'static str, kind: SyntaxKind) -> Node {
        let file = parse_source_file(
            &SourceFileParseOptions {
                file_name: "/a.ts".to_string(),
                path: Path("/a.ts".to_string()),
                ..Default::default()
            },
            text,
            ScriptKind::TS,
        )
        .root;
        let mut found = Node::NIL;
        for_each_descendant_of_kind(file, kind, &mut |n| found = n);
        assert!(!found.is_nil(), "no {kind:?} in {text:?}");
        found
    }

    // Go pseudochecker/lookup.go:21, :64 and :506 call
    // `debug.FailBadSyntaxKind` for a node the pseudochecker cannot type.
    #[test]
    fn bad_node_kinds_are_go_debug_failures() {
        let pc = new_pseudo_checker(true, false);
        let symbols = SymbolArena::new();
        let ident = node_of_kind("x;", SyntaxKind::Identifier);
        assert_eq!(
            go_panic_text(|| {
                pc.get_return_type_of_signature(&symbols, ident);
            }),
            "Debug failure. Node needs to be an inferrable node\nNode KindIdentifier was unexpected."
        );
        assert_eq!(
            go_panic_text(|| {
                pc.get_type_of_declaration(&symbols, ident);
            }),
            "Debug failure. node needs to be an inferrable node\nNode KindIdentifier was unexpected."
        );
        let prefix = node_of_kind("-x;", SyntaxKind::PrefixUnaryExpression);
        assert_eq!(
            go_panic_text(|| {
                pc.type_from_primitive_literal_prefix(prefix);
            }),
            "Debug failure. Unexpected node.\nNode KindIdentifier was unexpected."
        );
    }
}
