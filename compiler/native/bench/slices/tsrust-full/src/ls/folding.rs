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

// Go: ls/folding.go:61 keyForFoldingRange
