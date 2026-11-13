use crate::format::prelude::*;

use crate::flags_macros::go_enum;
use crate::frontend::scanner::scanner_p1::{};
use crate::frontend::scanner::{};

// Go: format/indent.go:17 GetIndentationForNode

// Go: format/indent.go:24 GetIndentation
// GetIndentation computes the expected indentation for a position in a source file.
// This is the Go port of SmartIndenter.getIndentation from TypeScript.

// Go: format/indent.go:111 getCommentIndent

// Go: format/indent.go:137 getLeadingCommentRangesOfNode
// PORT: Go returns an `iter.Seq` (nil for JSX text); this returns a Vec
// (empty for JSX text).

// Go: format/indent.go:144 getRangeOfEnclosingComment

// Go: format/indent.go:177 getBlockIndent

// Go: format/indent.go:193 getActualIndentationForListItemBeforeComma

// Go: format/indent.go:209 nextTokenKind
go_enum!(NextTokenKind, i32 {
    UNKNOWN = 0; // nextTokenKindUnknown
    OPEN_BRACE = 1; // nextTokenKindOpenBrace
    CLOSE_BRACE = 2; // nextTokenKindCloseBrace
});

// Go: format/indent.go:217 nextTokenIsCurlyBraceOnSameLineAsCursor

// Go: format/indent.go:238 getSmartIndent

// Go: format/indent.go:279 getIndentationForNodeWorker

// Go: format/indent.go:378 getActualIndentationForNode
/*
 * Function returns -1 if actual indentation for node should not be used (i.e because node is nested expression)
 */

// Go: format/indent.go:391 isArgumentAndStartLineOverlapsExpressionBeingCalled

// Go: format/indent.go:400 getActualIndentationForListItem

// Go: format/indent.go:427 getActualIndentationForListStartLine

// Go: format/indent.go:435 deriveActualIndentationFromList

// Go: format/indent.go:460 findColumnForFirstNonWhitespaceCharacterInLine

// Go: format/indent.go:465 FindFirstNonWhitespaceColumn

// Go: format/indent.go:477 findFirstNonWhitespaceCharacterAndColumn

// Go: format/indent.go:500 childStartsOnTheSameLineWithElseInIfStatement

// Go: format/indent.go:510 getStartLineAndCharacterForNode

// Go: format/indent.go:514 getStartLineForNode

// Go: format/indent.go:518 GetContainingList

// Go: format/indent.go:525 getListByPosition

// Go: format/indent.go:532 getListByRange

// Go: format/indent.go:579 getList

// Go: format/indent.go:589 getVisualListRange

// Go: format/indent.go:613 getContainingListOrParentStart

// Go: format/indent.go:624 isControlFlowEndingStatement
pub fn is_control_flow_ending_statement(kind: SyntaxKind, parent_kind: SyntaxKind) -> bool {
    match kind {
        SyntaxKind::ReturnStatement
        | SyntaxKind::ThrowStatement
        | SyntaxKind::ContinueStatement
        | SyntaxKind::BreakStatement => parent_kind != SyntaxKind::Block,
        _ => false,
    }
}

// Go: format/indent.go:637 ShouldIndentChildNode

// Go: format/indent.go:646 NodeWillIndentChild

// Go: format/indent.go:779 childIsUnindentedBranchOfConditionalExpression
// A multiline conditional typically increases the indentation of its whenTrue and whenFalse children:
//
// condition
//
//	? whenTrue
//	: whenFalse;
//
// However, that indentation does not apply if the subexpressions themselves span multiple lines,
// applying their own indentation:
//
//	(() => {
//	  return complexCalculationForCondition();
//	})() ? {
//
//	  whenTrue: 'multiline object literal'
//	} : (
//
//	whenFalse('multiline parenthesized expression')
//
// );
//
// In these cases, we must discard the indentation increase that would otherwise be applied to the
// whenTrue and whenFalse children to avoid double-indenting their contents. To identify this scenario,
// we check for the whenTrue branch beginning on the line that the condition ends, and the whenFalse
// branch beginning on the line that the whenTrue branch ends.

// Go: format/indent.go:800 argumentStartsOnSameLineAsPreviousArgument
