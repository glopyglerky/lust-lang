//! Port of Go `ls/change/tracker.go`.

use crate::ls::change::prelude::*;

use crate::flags_macros::go_enum;
use crate::spanmap::{self, Feature, SpanMap};

// Go: ls/change/tracker.go:21 NodeOptions
// PORT: Go `indentation *int` and `delta *int` are `Option<i32>`. The
// embedded `LeadingTriviaOption` and `TrailingTriviaOption` are the fields
// `leading_trivia_option` and `trailing_trivia_option`. Go unexported fields
// are `pub(crate)`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NodeOptions {
    /// Text to be inserted before the new node
    pub prefix: String,

    /// Text to be inserted after the new node
    pub suffix: String,

    /// Text of inserted node will be formatted with this indentation, otherwise indentation will be inferred from the old node
    pub(crate) indentation: Option<i32>,

    /// Text of inserted node will be formatted with this delta, otherwise delta will be inferred from the new node kind
    pub(crate) delta: Option<i32>,

    pub leading_trivia_option: LeadingTriviaOption,
    pub trailing_trivia_option: TrailingTriviaOption,
    pub(crate) joiner: String,
}

// Go: ls/change/tracker.go:39 LeadingTriviaOption
go_enum!(LeadingTriviaOption, i32 {
    NONE = 0; // LeadingTriviaOptionNone
    EXCLUDE = 1; // LeadingTriviaOptionExclude
    INCLUDE_ALL = 2; // LeadingTriviaOptionIncludeAll
    JS_DOC = 3; // LeadingTriviaOptionJSDoc
    START_LINE = 4; // LeadingTriviaOptionStartLine
});

// Go: ls/change/tracker.go:49 TrailingTriviaOption
go_enum!(TrailingTriviaOption, i32 {
    NONE = 0; // TrailingTriviaOptionNone
    EXCLUDE = 1; // TrailingTriviaOptionExclude
    EXCLUDE_WHITESPACE = 2; // TrailingTriviaOptionExcludeWhitespace
    INCLUDE = 3; // TrailingTriviaOptionInclude
});

// Go: ls/change/tracker.go:58 trackerEditKind
go_enum!(TrackerEditKind, i32 {
    TEXT = 1; // trackerEditKindText
    REMOVE = 2; // trackerEditKindRemove
    REPLACE_WITH_SINGLE_NODE = 3; // trackerEditKindReplaceWithSingleNode
    REPLACE_WITH_MULTIPLE_NODES = 4; // trackerEditKindReplaceWithMultipleNodes
});

// Go: ls/change/tracker.go:67 trackerEdit
// PORT: Go embeds `core.TextRange` and `*ast.Node`; they are the fields
// `text_range` and `node`. Go stores `*trackerEdit` in the change map; here
// the map owns the edits.
#[derive(Clone, Debug, Default)]
pub struct TrackerEdit {
    pub kind: TrackerEditKind,
    pub text_range: TextRange,

    pub new_text: String, // kind == text

    pub node: Node,       // single
    pub nodes: Vec<Node>, // multiple
    pub options: NodeOptions,
}

// Go: ls/change/tracker.go:78 nodesInsertedAtStartState
#[derive(Clone, Copy, Debug)]
pub struct NodesInsertedAtStartState {
    pub node: Node,
    pub source_file: Node,
}

// Go: ls/change/tracker.go:83 Tracker
// PORT: Go embeds `*printer.EmitContext` and `*ast.NodeFactory`. The context
// is the field `emit_context`; the factory (Go
// `&emitContext.Factory.NodeFactory`) is the method `node_factory()`. Go
// promoted calls such as `t.NewToken(..)` are `t.node_factory().new_token(..)`.
// Go unexported fields are `pub(crate)`.

// Go: ls/change/tracker.go:107 deletedNode
#[derive(Clone, Copy, Debug)]
pub struct DeletedNode {
    pub source_file: Node,
    pub node: Node,
}

// Go: ls/change/tracker.go:112 NewTracker

// PORT: methods that add changes take `&mut self` (Go mutates through the
// `*Tracker` pointer). Go `core.TextPos` positions are `i32`.

// Go: ls/change/tracker.go:788 getMembersOrProperties

// Go: ls/change/tracker.go:795 rangeContainsRangeExclusive

// Go: ls/change/tracker.go:799 isSeparator

// Go: ls/change/tracker.go:803 findIndentationColumn
fn find_indentation_column(text: &str, line_start: i32, member_start: i32, tab_size: i32) -> i32 {
    let mut column: i32 = 0;

    let bytes = text.as_bytes();
    let mut i = line_start;
    while i < member_start && (i as usize) < bytes.len() {
        // PORT: Go `rune(text[i])` converts one byte, as `char::from(u8)`.
        let ch = char::from(bytes[i as usize]);

        if is_line_break(ch) {
            return -1;
        }
        if is_white_space_single_line(ch) {
            column = advance_indentation_column(column, ch, tab_size);
            i += 1;
            continue;
        }
        return column;
    }

    column
}

// Go: ls/change/tracker.go:822 advanceIndentationColumn
fn advance_indentation_column(column: i32, ch: char, tab_size: i32) -> i32 {
    if ch == '\t' {
        return column + tab_size - (column % tab_size);
    }
    column + 1
}
