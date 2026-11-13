use crate::ls::prelude::*;

use crate::spanmap::{Feature, Fidelity};

// Go `internal/ls/folding.go`: textDocument/foldingRange.



// Go: ls/folding.go:203 visitNode

// Go: ls/folding.go:295 addOutliningForLeadingCommentsForNode

// Go: ls/folding.go:302 addOutliningForLeadingCommentsForPos

// Go: ls/folding.go:367 regionDelimiterResult
struct RegionDelimiterResult {
    is_start: bool,
    name: String,
}

// Go: ls/folding.go:372 parseRegionDelimiter

// Go: ls/folding.go:400 getOutliningSpanForNode

// Go: ls/folding.go:464 spanForImportExportElements

// Go: ls/folding.go:485 spanForParenthesizedExpression

// Go: ls/folding.go:497 spanForCallExpression

// Go: ls/folding.go:510 spanForArrowFunction

// Go: ls/folding.go:522 spanForTemplateLiteral

// Go: ls/folding.go:529 spanForJSXElement
// PORT: Go returns `*FoldingRange`; nil is `None`.

// Go: ls/folding.go:549 spanForJSXAttributes

// Go: ls/folding.go:562 spanForNodeArray

// Go: ls/folding.go:573 spanForNode

// Go: ls/folding.go:586 rangeBetweenTokens
// PORT: Go returns `*FoldingRange`; nil is `None`.

// Go: ls/folding.go:600 supportsCollapsedText

// Go: ls/folding.go:604 createFoldingRange
// PORT: Go returns a `*FoldingRange` that is never nil, so this returns the
// value. The Go empty kind `""` is `FoldingRangeKind::default()`.

// Go: ls/folding.go:622 createFoldingRangeFromBounds
// PORT: Go returns `*FoldingRange`; nil is `None`.


// Go: ls/folding.go:634 functionSpan

// Go: ls/folding.go:643 tryGetFunctionOpenToken

// Go: ls/folding.go:653 isNodeArrayMultiLine

// Go: ls/folding.go:53 foldingRangeKey
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct FoldingRangeKey {
    start_line: u32,
    start_character: u32,
    end_line: u32,
    end_character: u32,
    kind: lsproto::FoldingRangeKind,
    collapsed_text: String,
    has_start_character: bool,
    has_end_character: bool,
    has_kind: bool,
    has_collapsed_text: bool,
}

// Go: ls/folding.go:61 keyForFoldingRange
fn key_for_folding_range(folding_range: &lsproto::FoldingRange) -> FoldingRangeKey {
    let mut key = FoldingRangeKey {
        start_line: folding_range.start_line,
        start_character: 0,
        end_line: folding_range.end_line,
        end_character: 0,
        kind: lsproto::FoldingRangeKind::default(),
        collapsed_text: String::new(),
        has_start_character: false,
        has_end_character: false,
        has_kind: false,
        has_collapsed_text: false,
    };
    if let Some(start_character) = folding_range.start_character {
        key.start_character = start_character;
        key.has_start_character = true;
    }
    if let Some(end_character) = folding_range.end_character {
        key.end_character = end_character;
        key.has_end_character = true;
    }
    if let Some(kind) = &folding_range.kind {
        key.kind = kind.clone();
        key.has_kind = true;
    }
    if let Some(collapsed_text) = &folding_range.collapsed_text {
        key.collapsed_text = collapsed_text.clone();
        key.has_collapsed_text = true;
    }
    key
}
