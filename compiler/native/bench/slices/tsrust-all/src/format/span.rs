use crate::format::prelude::*;

use crate::flags_macros::go_enum;
use crate::frontend::core_textchange::TextChange;
use crate::frontend::scanner::scanner_p1::{};
use std::cell::Cell;
use std::sync::Arc;

// Go: format/span.go:20 findEnclosingNode

// Go: format/span.go:51 getScanStartPosition
/*
 * Start of the original range might fall inside the comment - scanner will not yield appropriate results
 * This function will look for token that is located before the start of target range
 * and return its end as start position for the scanner.
 */

// Go: format/span.go:89 getOwnOrInheritedDelta
/*
 * For cases like
 * if (a ||
 *     b ||$
 *     c) {...}
 * If we hit Enter at $ we want line '    b ||' to be indented.
 * Formatting will be applied to the last two lines.
 * Node that fully encloses these lines is binary expression 'a ||...'.
 * Initial indentation for this node will be 0.
 * Binary expressions don't introduce new indentation scopes, however it is possible
 * that some parent node on the same line does - like if statement in this case.
 * Note that we are considering parents only from the same line with initial node -
 * if parent is on the different line - its delta was already contributed
 * to the initial indentation.
 */

// Go: format/span.go:109 rangeHasNoErrors
pub fn range_has_no_errors(_: TextRange) -> bool {
    false
}

// Go: format/span.go:113 prepareRangeContainsErrorFunction
// PORT: Go returns `func(r core.TextRange) bool`. The closure advances
// `index`, so it is `FnMut`. It keeps the ranges of the errors it picks,
// not the errors, so it does not borrow `errors` (a file version guard).

// Go: format/span.go:155 formatSpanWorker
// PORT: Go keeps the NodeVisitor built in `execute` in `w.visitor`. The Rust
// visitor holds `&mut FormatSpanWorker<'t>` as its ctx, so it cannot live in the
// worker; `execute_process_node_visitor` builds one per call with the same
// callbacks (`new_format_span_visitor`). The worker owns the formatting
// scanner (see `new_formatting_scanner`), which borrows the file text
// (`'t`).

// Go: format/span.go:188 newFormatSpanWorker
// PORT: `range_contains_error` is 'static: every caller passes a closure over
// the file's parse diagnostics (`&'static`) or none.

// Go: format/span.go:211 getNonDecoratorTokenPosOfNode


// Go: format/span.go:231 (the visitor built in execute)
// PORT: see `FormatSpanWorker`. Go passes `&ast.NodeFactory{}`; `None` is the
// visitor's own default factory.


// Go: format/span.go:754 LineAction
go_enum!(LineAction, i32 {
    NONE = 0; // LineActionNone
    LINE_ADDED = 1; // LineActionLineAdded
    LINE_REMOVED = 2; // LineActionLineRemoved
});


// Go: format/span.go:870 isStringOrRegularExpressionOrTemplateLiteral
pub fn is_string_or_regular_expression_or_template_literal(kind: SyntaxKind) -> bool {
    kind == SyntaxKind::StringLiteral
        || kind == SyntaxKind::RegularExpressionLiteral
        || is_template_literal_kind(kind)
}

// Go: format/span.go:874 isComment
pub fn is_comment(kind: SyntaxKind) -> bool {
    kind == SyntaxKind::SingleLineCommentTrivia || kind == SyntaxKind::MultiLineCommentTrivia
}


/// Go `strings.Repeat(s, count)`.
// PORT: Go panics on a negative count; `str::repeat` takes a usize.
fn strings_repeat(s: &str, count: i32) -> String {
    if count < 0 {
        panic!("strings: negative Repeat count");
    }
    s.repeat(count as usize)
}

// Go: format/span.go:998 getIndentationString
pub fn get_indentation_string(indentation: i32, options: &lsutil::FormatCodeSettings) -> String {
    // go's `strings.Repeat` already has static, global caching for repeated tabs and spaces, so there's no need to cache here like in strada
    if !options.editor_settings.convert_tabs_to_spaces.is_true() {
        if options.editor_settings.tab_size == 0 {
            return String::new();
        }
        let tabs =
            (f64::from(indentation) / f64::from(options.editor_settings.tab_size)).floor() as i32;
        let spaces = indentation - (tabs * options.editor_settings.tab_size);
        let mut res = strings_repeat("\t", tabs);
        if spaces > 0 {
            res = res + &strings_repeat(" ", spaces);
        }

        res
    } else {
        strings_repeat(" ", indentation)
    }
}

// Go: format/span.go:1017 createTextChangeFromStartLength
pub fn create_text_change_from_start_length(start: i32, length: i32, new_text: &str) -> TextChange {
    TextChange {
        new_text: new_text.to_string(),
        text_range: TextRange::new(start, start + length),
    }
}


// Go: format/span.go:1121 dynamicIndenter
// PORT: Go shares `*dynamicIndenter` between the worker and its callers and
// mutates `indentation` and `delta` through it (recomputeIndentation). Here
// it is shared as `Rc<DynamicIndenter>` with those two fields in `Cell`s. It
// holds no reference to the worker, so there is no cycle.
pub struct DynamicIndenter {
    pub node: Node,
    pub node_start_line: i32,
    pub indentation: Cell<i32>,
    pub delta: Cell<i32>,

    pub options: lsutil::FormatCodeSettings,
    pub source_file: Node,
}

impl DynamicIndenter {
    // Go: format/span.go:1131 getIndentationForComment

    // Go: format/span.go:1159 getIndentationForToken
    // if list end token is LessThanToken '>' then its delta should be explicitly suppressed
    // so that LessThanToken as a binary operator can still be indented.
    // foo.then
    //
    //	<
    //	    number,
    //	    string,
    //	>();
    //
    // vs
    // var a = xValue
    //
    //	> yValue;

    // Go: format/span.go:1166 getIndentation
    pub fn get_indentation(&self) -> i32 {
        self.indentation.get()
    }

    // Go: format/span.go:1170 getDelta

    // Go: format/span.go:1178 recomputeIndentation

    // Go: format/span.go:1193 shouldAddDelta
}

// Go: format/span.go:1214 getFirstNonDecoratorTokenOfNode

