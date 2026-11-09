use crate::ls::prelude::*;

// Port of Go `ls/codeactions_fixmissingtypeannotation.go`.
//
// PORT (whole file):
// - Go `isolatedDeclarationsFixer` holds pointers to the checker and the
//   change tracker, and the caller keeps using the tracker afterwards. The
//   Rust fixer borrows both (`&'a mut`).
// - Go `fixedNodes map[*ast.Node]bool` is `FxHashSet<Node>`;
//   `symbolsToImport` keeps its order (`Vec<SymbolId>`).
// - Go `autoimport.ImportAdder` (nil allowed) is borrowed as
//   `Option<&mut dyn autoimport::ImportAdder>`; the owner holds
//   `Option<Box<dyn autoimport::ImportAdder>>` (w3 shape).
// - Go `f.changeTracker.NodeFactory` is `change_tracker.node_factory()`.
//   Go passes the factory to some helpers as a parameter; in Rust a
//   factory borrowed from the tracker cannot live across a `&mut self`
//   call, so those helpers read it from the tracker again (same factory).
// - Go reads `*ast.Symbol` and `*checker.Type` fields directly. Here they
//   are read from the checker arenas (`checker.sym(s)`, `checker.ty(t)`).

use crate::flags_macros::go_enum;
use std::sync::LazyLock;

// Go: ls/codeactions_fixmissingtypeannotation.go:21 isolatedDeclarationsFixErrorCodes

// Go: ls/codeactions_fixmissingtypeannotation.go:45 fixMissingTypeAnnotationOnExportsFixID
const FIX_MISSING_TYPE_ANNOTATION_ON_EXPORTS_FIX_ID: &str = "fixMissingTypeAnnotationOnExports";

// Go: ls/codeactions_fixmissingtypeannotation.go:48 IsolatedDeclarationsFixProvider

// Go: ls/codeactions_fixmissingtypeannotation.go:56 canHaveTypeAnnotationKinds
// canHaveTypeAnnotationKinds are the node kinds that can have type annotations added.
static CAN_HAVE_TYPE_ANNOTATION_KINDS: LazyLock<FxHashMap<SyntaxKind, bool>> =
    LazyLock::new(|| {
        let mut kinds: FxHashMap<SyntaxKind, bool> = FxHashMap::default();
        kinds.insert(SyntaxKind::GetAccessor, true);
        kinds.insert(SyntaxKind::MethodDeclaration, true);
        kinds.insert(SyntaxKind::PropertyDeclaration, true);
        kinds.insert(SyntaxKind::FunctionDeclaration, true);
        kinds.insert(SyntaxKind::FunctionExpression, true);
        kinds.insert(SyntaxKind::ArrowFunction, true);
        kinds.insert(SyntaxKind::VariableDeclaration, true);
        kinds.insert(SyntaxKind::Parameter, true);
        kinds.insert(SyntaxKind::ExportAssignment, true);
        kinds.insert(SyntaxKind::ClassDeclaration, true);
        kinds.insert(SyntaxKind::ObjectBindingPattern, true);
        kinds.insert(SyntaxKind::ArrayBindingPattern, true);
        kinds
    });

// Go: ls/codeactions_fixmissingtypeannotation.go:72 declarationEmitNodeBuilderFlags
// declarationEmitNodeBuilderFlags are the node builder flags used for declaration emit.

// Go: ls/codeactions_fixmissingtypeannotation.go:80 typePrintMode
go_enum!(TypePrintMode, i32 {
    FULL = 0; // typePrintModeFull
    RELATIVE = 1; // typePrintModeRelative: typeof X
    WIDENED = 2; // typePrintModeWidened: widened literal type
});

// Go: ls/codeactions_fixmissingtypeannotation.go:88 getIsolatedDeclarationsCodeActions

// Go: ls/codeactions_fixmissingtypeannotation.go:128 getAllIsolatedDeclarationsCodeActions

// Go: ls/codeactions_fixmissingtypeannotation.go:168 tryCodeAction

// Go: ls/codeactions_fixmissingtypeannotation.go:216 isolatedDeclarationsFixer
// isolatedDeclarationsFixer encapsulates the state for fixing isolated declarations errors.


// Go: ls/codeactions_fixmissingtypeannotation.go:309 needsParenthesizedExpressionForAssertion
// needsParenthesizedExpressionForAssertion checks if an expression needs parentheses for an assertion.

// Go: ls/codeactions_fixmissingtypeannotation.go:314 createAsExpression
// createAsExpression creates an `expr as Type` expression, parenthesizing if needed.


// Go: ls/codeactions_fixmissingtypeannotation.go:480 isExpandoPropertyDeclarationForFix
// findExpandoFunction finds the function declaration that has expando properties assigned to it.
// isExpandoPropertyDeclarationForFix matches TS's isExpandoPropertyDeclaration which includes
// PropertyAccessExpression, ElementAccessExpression, and BinaryExpression. The shared
// ast.IsExpandoPropertyDeclaration was narrowed to BinaryExpression only for checker purposes.

// Go: ls/codeactions_fixmissingtypeannotation.go:484 findExpandoFunction


// Go: ls/codeactions_fixmissingtypeannotation.go:1108 isConstAssertion
// isConstAssertion checks if a node is an `as const` or `<const>` assertion.


// Go: ls/codeactions_fixmissingtypeannotation.go:1209 endOfRequiredTypeParameters
// endOfRequiredTypeParameters finds the number of type arguments that are
// actually required (i.e., differ from their defaults). Ported from TS's
// services/codefixes/helpers.ts endOfRequiredTypeParameters.

// Go: ls/codeactions_fixmissingtypeannotation.go:1246 typeParamHasDefault
// typeParamHasDefault checks if a type parameter has a default type declaration.
// PORT: `ch` is added to read the type and symbol arenas.


// Go: ls/codeactions_fixmissingtypeannotation.go:1282 typeToStringForDiag
// typeToStringForDiag converts a type node to a string for use in diagnostic descriptions.
// It reuses the change tracker's EmitContext so that generated identifier names are resolved
// consistently with the actual code edits, and passes the source file so that the printer's
// name generator can check for conflicts with existing file-level identifiers.

// Go: ls/codeactions_fixmissingtypeannotation.go:1305 findAncestorWithMissingType
// findAncestorWithMissingType walks up the ancestor chain to find a node that
// can have a type annotation and is missing one.

// Go: ls/codeactions_fixmissingtypeannotation.go:1318 findBestFittingNode
// findBestFittingNode walks up from the token to find the node that best fits the diagnostic span.

// Go: ls/codeactions_fixmissingtypeannotation.go:1340 isNamedDeclarationKind
// isNamedDeclarationKind matches TS's isDeclarationKind, which is narrower than Go's IsDeclaration.
// Go's IsDeclaration returns true for any node with DeclarationData (including CallExpression),
// while TS's isDeclaration only returns true for specific named declaration kinds.

// Go: ls/codeactions_fixmissingtypeannotation.go:1360 isValueSignatureDeclaration
// isValueSignatureDeclaration checks if a node is a function-like declaration that produces a value.

// Go: ls/codeactions_fixmissingtypeannotation.go:1368 getIdentifierNameForNode
// getIdentifierNameForNode derives a meaningful variable name from a node expression.
// For property access expressions like `obj.foo`, returns "foo". Otherwise returns "newLocal".
// Ported from TS's getIdentifierForNode in services/refactors/helpers.ts.

