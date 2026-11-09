//! Port of Go `transformers/utilities.go`.

use crate::prelude::*;

// Go: transformers/utilities.go:12 IsGeneratedIdentifier

// Go: transformers/utilities.go:16 IsHelperName

// Go: transformers/utilities.go:20 IsLocalName

// Go: transformers/utilities.go:24 IsExportName

// Go: transformers/utilities.go:28 IsIdentifierReference

// Go: transformers/utilities.go:112 convertBindingElementToArrayAssignmentElement

// Go: transformers/utilities.go:135 convertBindingElementToObjectAssignmentElement

// Go: transformers/utilities.go:169 ConvertBindingPatternToAssignmentPattern

// Go: transformers/utilities.go:180 convertBindingElementToObjectAssignmentPattern

// Go: transformers/utilities.go:193 convertBindingElementToArrayAssignmentPattern

// Go: transformers/utilities.go:206 convertBindingNameToAssignmentElementTarget

// Go: transformers/utilities.go:213 ConvertVariableDeclarationToAssignmentExpression

// Go: transformers/utilities.go:224 SingleOrMany
// PORT: Go distinguishes a nil slice (result nil) from an empty one (an
// empty SyntaxList). `None` is the nil slice.

// Go: transformers/utilities.go:241 IsSimpleCopiableExpression
// Used in the module transformer to check if an expression is reasonably without sideeffect,
//
//	and thus better to copy into multiple places rather than to cache in a temporary variable
//	- this is mostly subjective beyond the requirement that the expression not be sideeffecting
//
// Also used by the logical assignment downleveling transform to skip temp variables when they're
// not needed.

// Go: transformers/utilities.go:248 IsOriginalNodeSingleLine

// Go: transformers/utilities.go:270 IsSimpleInlineableExpression

// Go: transformers/utilities.go:275 FindSuperStatementIndexPath
// FindSuperStatementIndexPath finds a path of indices to a statement containing a `super()` call.
// PORT: Go returns a nil slice when there is no `super()` call; that is an
// empty `Vec` here (Go callers only check `len`).

// Go: transformers/utilities.go:281 findSuperStatementIndexPathWorker

// Go: transformers/utilities.go:296 GetSuperCallFromStatement
// GetSuperCallFromStatement extracts the super() call expression from an expression statement, if any.

// Go: transformers/utilities.go:308 MoveRangePastModifiers
// MoveRangePastModifiers returns a text range that starts past any modifiers on the node.

// Go: transformers/utilities.go:325 MoveRangePastDecorators
// MoveRangePastDecorators returns a text range that starts past any decorators on the node.

// Go: transformers/utilities.go:341 GetNonAssignmentOperatorForCompoundAssignment
// GetNonAssignmentOperatorForCompoundAssignment returns the non-assignment operator for a compound assignment.
pub fn get_non_assignment_operator_for_compound_assignment(kind: SyntaxKind) -> SyntaxKind {
    match kind {
        SyntaxKind::PlusEqualsToken => SyntaxKind::PlusToken,
        SyntaxKind::MinusEqualsToken => SyntaxKind::MinusToken,
        SyntaxKind::AsteriskEqualsToken => SyntaxKind::AsteriskToken,
        SyntaxKind::AsteriskAsteriskEqualsToken => SyntaxKind::AsteriskAsteriskToken,
        SyntaxKind::SlashEqualsToken => SyntaxKind::SlashToken,
        SyntaxKind::PercentEqualsToken => SyntaxKind::PercentToken,
        SyntaxKind::LessThanLessThanEqualsToken => SyntaxKind::LessThanLessThanToken,
        SyntaxKind::GreaterThanGreaterThanEqualsToken => SyntaxKind::GreaterThanGreaterThanToken,
        SyntaxKind::GreaterThanGreaterThanGreaterThanEqualsToken => {
            SyntaxKind::GreaterThanGreaterThanGreaterThanToken
        }
        SyntaxKind::AmpersandEqualsToken => SyntaxKind::AmpersandToken,
        SyntaxKind::BarEqualsToken => SyntaxKind::BarToken,
        SyntaxKind::CaretEqualsToken => SyntaxKind::CaretToken,
        SyntaxKind::BarBarEqualsToken => SyntaxKind::BarBarToken,
        SyntaxKind::AmpersandAmpersandEqualsToken => SyntaxKind::AmpersandAmpersandToken,
        SyntaxKind::QuestionQuestionEqualsToken => SyntaxKind::QuestionQuestionToken,
        _ => kind,
    }
}
