//! Port of printer/printer.go lines 4692 to 6280: lists, general entry
//! points (Emit, Write), comments, source map stubs, name generation,
//! scoped operations, token emit flags and list formats.
//!
//! PORT: this part assumes these `Printer` field types from printer_p1.rs:
//! - `writer` and `own_writer`: `Option<Rc<RefCell<dyn EmitTextWriter>>>`
//!   (Go interface values are shared; the caller of `Write` keeps its writer).
//! - `print_handlers: PrintHandlers` (Go embedding), with handler fields
//!   `Option<Rc<dyn Fn(..)>>`.
//! - `current_source_file`, `external_helpers_module_name`: `Node`.
//! - `source_map_source`, `most_recent_source_map_source`: `SourceMapSource`.
//! - `unique_helper_names`: `Option<FxHashMap<String, Node>>` (Go nil map).
//! - `detached_comments_info`: `Vec<DetachedCommentsInfo>` (Go core.Stack).
//! - `source_map_generator`: `Option<Rc<RefCell<SourceMapGenerator>>>`.
//! - `source_map_source_index`, `most_recent_source_map_source_index`: `i32`.
//! - `source_map_line_char_cache`: an `Option<..>` (Go pointer).
//! - Go `*commentState` / `*sourceMapState` from the arenas are
//!   `Option<CommentState>` / `Option<SourceMapState>` values. The arenas
//!   only avoid Go heap allocation, so they are not used here.

use crate::flags_macros::{go_enum, go_flags};
use crate::gostd::debug::kind_string;
use crate::prelude::*;
use crate::printer::*;

thread_local! {
    /// Go `p.emitContext.Factory.AsNodeFactory()` for the comment range
    /// scanner. The scanner only uses it to build `CommentRange` values.
    static COMMENT_RANGE_FACTORY: NodeFactory = NodeFactory::default();
}

// Go: scanner/scanner.go:2799 GetLeadingCommentRanges
// PORT: the free functions below have no printer, so they use a plain
// `ast` factory; the scanner does not allocate nodes with it.

// Go: scanner/scanner.go:2803 GetTrailingCommentRanges


// Go: printer/printer.go:5515 formatSynthesizedComment
pub(crate) fn format_synthesized_comment(comment: &SynthesizedComment) -> String {
    if comment.kind == SyntaxKind::MultiLineCommentTrivia {
        return format!("/*{}*/", comment.text);
    }
    format!("//{}", comment.text)
}

// Go: printer/printer.go:5757 commentSeparator
go_enum!(CommentSeparator, u32 {
    NONE = 0; // commentSeparatorNone
    BEFORE = 1; // commentSeparatorBefore
    AFTER = 2; // commentSeparatorAfter
});

// Go: printer/printer.go:6201 tokenEmitFlags
go_flags!(TokenEmitFlags, u32 {
    NO_COMMENTS = 1 << 0; // tefNoComments
    INDENT_LEADING_COMMENTS = 1 << 1; // tefIndentLeadingComments
    NO_SOURCE_MAPS = 1 << 2; // tefNoSourceMaps

    NONE = 0; // tefNone
});

// Go: printer/printer.go:6223 ListFormat
go_flags!(ListFormat, i32 {
    NONE = 0; // LFNone

    // Line separators
    SINGLE_LINE = 0; // Prints the list on a single line (default).
    MULTI_LINE = 1 << 0; // Prints the list on multiple lines.
    PRESERVE_LINES = 1 << 1; // Prints the list using line preservation if possible.
    LINES_MASK = ListFormat::SINGLE_LINE.0 | ListFormat::MULTI_LINE.0 | ListFormat::PRESERVE_LINES.0;

    // Delimiters
    NOT_DELIMITED = 0; // There is no delimiter between list items (default).
    BAR_DELIMITED = 1 << 2; // Each list item is space-and-bar (" |") delimited.
    AMPERSAND_DELIMITED = 1 << 3; // Each list item is space-and-ampersand (" &") delimited.
    COMMA_DELIMITED = 1 << 4; // Each list item is comma (",") delimited.
    ASTERISK_DELIMITED = 1 << 5; // Each list item is asterisk ("\n *") delimited, used with JSDoc.
    DELIMITERS_MASK = ListFormat::BAR_DELIMITED.0 | ListFormat::AMPERSAND_DELIMITED.0 | ListFormat::COMMA_DELIMITED.0 | ListFormat::ASTERISK_DELIMITED.0;

    ALLOW_TRAILING_COMMA = 1 << 6; // Write a trailing comma (",") if present.

    // Whitespace
    INDENTED = 1 << 7; // The list should be indented.
    SPACE_BETWEEN_BRACES = 1 << 8; // Inserts a space after the opening brace and before the closing brace.
    SPACE_BETWEEN_SIBLINGS = 1 << 9; // Inserts a space between each sibling node.

    // Brackets/Braces
    BRACES = 1 << 10; // The list is surrounded by "{" and "}".
    PARENTHESIS = 1 << 11; // The list is surrounded by "(" and ")".
    ANGLE_BRACKETS = 1 << 12; // The list is surrounded by "<" and ">".
    SQUARE_BRACKETS = 1 << 13; // The list is surrounded by "[" and "]".
    BRACKETS_MASK = ListFormat::BRACES.0 | ListFormat::PARENTHESIS.0 | ListFormat::ANGLE_BRACKETS.0 | ListFormat::SQUARE_BRACKETS.0;

    OPTIONAL_IF_NIL = 1 << 14; // Do not emit brackets if the list is nil.
    OPTIONAL_IF_EMPTY = 1 << 15; // Do not emit brackets if the list is empty.
    OPTIONAL = ListFormat::OPTIONAL_IF_NIL.0 | ListFormat::OPTIONAL_IF_EMPTY.0;

    // Other
    PREFER_NEW_LINE = 1 << 16; // Prefer adding a LineTerminator between synthesized nodes.
    NO_TRAILING_NEW_LINE = 1 << 17; // Do not emit a trailing NewLine for a MultiLine list.
    NO_INTERVENING_COMMENTS = 1 << 18; // Do not emit comments between each node
    NO_SPACE_IF_EMPTY = 1 << 19; // If the literal is empty, do not add spaces between braces.
    SINGLE_ELEMENT = 1 << 20;
    SPACE_AFTER_LIST = 1 << 21; // Add space after list

    // Precomputed Formats
    MODIFIERS = ListFormat::SINGLE_LINE.0 | ListFormat::SPACE_BETWEEN_SIBLINGS.0 | ListFormat::NO_INTERVENING_COMMENTS.0 | ListFormat::SPACE_AFTER_LIST.0;
    HERITAGE_CLAUSES = ListFormat::SINGLE_LINE.0 | ListFormat::SPACE_BETWEEN_SIBLINGS.0;
    SINGLE_LINE_TYPE_LITERAL_MEMBERS = ListFormat::SINGLE_LINE.0 | ListFormat::SPACE_BETWEEN_BRACES.0 | ListFormat::SPACE_BETWEEN_SIBLINGS.0;
    MULTI_LINE_TYPE_LITERAL_MEMBERS = ListFormat::MULTI_LINE.0 | ListFormat::INDENTED.0 | ListFormat::OPTIONAL_IF_EMPTY.0;

    SINGLE_LINE_TUPLE_TYPE_ELEMENTS = ListFormat::COMMA_DELIMITED.0 | ListFormat::SPACE_BETWEEN_SIBLINGS.0 | ListFormat::SINGLE_LINE.0;
    MULTI_LINE_TUPLE_TYPE_ELEMENTS = ListFormat::COMMA_DELIMITED.0 | ListFormat::INDENTED.0 | ListFormat::SPACE_BETWEEN_SIBLINGS.0 | ListFormat::MULTI_LINE.0;
    UNION_TYPE_CONSTITUENTS = ListFormat::BAR_DELIMITED.0 | ListFormat::SPACE_BETWEEN_SIBLINGS.0 | ListFormat::SINGLE_LINE.0;
    INTERSECTION_TYPE_CONSTITUENTS = ListFormat::AMPERSAND_DELIMITED.0 | ListFormat::SPACE_BETWEEN_SIBLINGS.0 | ListFormat::SINGLE_LINE.0;
    OBJECT_BINDING_PATTERN_ELEMENTS = ListFormat::SINGLE_LINE.0 | ListFormat::ALLOW_TRAILING_COMMA.0 | ListFormat::SPACE_BETWEEN_BRACES.0 | ListFormat::COMMA_DELIMITED.0 | ListFormat::SPACE_BETWEEN_SIBLINGS.0 | ListFormat::NO_SPACE_IF_EMPTY.0;
    ARRAY_BINDING_PATTERN_ELEMENTS = ListFormat::SINGLE_LINE.0 | ListFormat::ALLOW_TRAILING_COMMA.0 | ListFormat::COMMA_DELIMITED.0 | ListFormat::SPACE_BETWEEN_SIBLINGS.0 | ListFormat::NO_SPACE_IF_EMPTY.0;
    OBJECT_LITERAL_EXPRESSION_PROPERTIES = ListFormat::PRESERVE_LINES.0 | ListFormat::COMMA_DELIMITED.0 | ListFormat::SPACE_BETWEEN_SIBLINGS.0 | ListFormat::SPACE_BETWEEN_BRACES.0 | ListFormat::INDENTED.0 | ListFormat::BRACES.0 | ListFormat::NO_SPACE_IF_EMPTY.0;
    IMPORT_ATTRIBUTES = ListFormat::PRESERVE_LINES.0 | ListFormat::COMMA_DELIMITED.0 | ListFormat::SPACE_BETWEEN_SIBLINGS.0 | ListFormat::SPACE_BETWEEN_BRACES.0 | ListFormat::INDENTED.0 | ListFormat::BRACES.0 | ListFormat::NO_SPACE_IF_EMPTY.0;
    ARRAY_LITERAL_EXPRESSION_ELEMENTS = ListFormat::PRESERVE_LINES.0 | ListFormat::COMMA_DELIMITED.0 | ListFormat::SPACE_BETWEEN_SIBLINGS.0 | ListFormat::ALLOW_TRAILING_COMMA.0 | ListFormat::INDENTED.0 | ListFormat::SQUARE_BRACKETS.0;
    COMMA_LIST_ELEMENTS = ListFormat::COMMA_DELIMITED.0 | ListFormat::SPACE_BETWEEN_SIBLINGS.0 | ListFormat::SINGLE_LINE.0;
    CALL_EXPRESSION_ARGUMENTS = ListFormat::COMMA_DELIMITED.0 | ListFormat::SPACE_BETWEEN_SIBLINGS.0 | ListFormat::SINGLE_LINE.0 | ListFormat::PARENTHESIS.0;
    NEW_EXPRESSION_ARGUMENTS = ListFormat::COMMA_DELIMITED.0 | ListFormat::SPACE_BETWEEN_SIBLINGS.0 | ListFormat::SINGLE_LINE.0 | ListFormat::PARENTHESIS.0 | ListFormat::OPTIONAL_IF_NIL.0;
    TEMPLATE_EXPRESSION_SPANS = ListFormat::SINGLE_LINE.0 | ListFormat::NO_INTERVENING_COMMENTS.0;
    SINGLE_LINE_BLOCK_STATEMENTS = ListFormat::SPACE_BETWEEN_BRACES.0 | ListFormat::SPACE_BETWEEN_SIBLINGS.0 | ListFormat::SINGLE_LINE.0;
    MULTI_LINE_BLOCK_STATEMENTS = ListFormat::INDENTED.0 | ListFormat::MULTI_LINE.0;
    VARIABLE_DECLARATION_LIST = ListFormat::COMMA_DELIMITED.0 | ListFormat::SPACE_BETWEEN_SIBLINGS.0 | ListFormat::SINGLE_LINE.0;
    SINGLE_LINE_FUNCTION_BODY_STATEMENTS = ListFormat::SINGLE_LINE.0 | ListFormat::SPACE_BETWEEN_SIBLINGS.0 | ListFormat::SPACE_BETWEEN_BRACES.0;
    MULTI_LINE_FUNCTION_BODY_STATEMENTS = ListFormat::MULTI_LINE.0;
    CLASS_HERITAGE_CLAUSES = ListFormat::SINGLE_LINE.0;
    CLASS_MEMBERS = ListFormat::INDENTED.0 | ListFormat::MULTI_LINE.0;
    INTERFACE_MEMBERS = ListFormat::INDENTED.0 | ListFormat::MULTI_LINE.0;
    ENUM_MEMBERS = ListFormat::COMMA_DELIMITED.0 | ListFormat::INDENTED.0 | ListFormat::MULTI_LINE.0;
    CASE_BLOCK_CLAUSES = ListFormat::INDENTED.0 | ListFormat::MULTI_LINE.0;
    NAMED_IMPORTS_OR_EXPORTS_ELEMENTS = ListFormat::COMMA_DELIMITED.0 | ListFormat::SPACE_BETWEEN_SIBLINGS.0 | ListFormat::ALLOW_TRAILING_COMMA.0 | ListFormat::SINGLE_LINE.0 | ListFormat::SPACE_BETWEEN_BRACES.0 | ListFormat::NO_SPACE_IF_EMPTY.0;
    JSX_ELEMENT_OR_FRAGMENT_CHILDREN = ListFormat::SINGLE_LINE.0 | ListFormat::NO_INTERVENING_COMMENTS.0;
    JSX_ELEMENT_ATTRIBUTES = ListFormat::SINGLE_LINE.0 | ListFormat::SPACE_BETWEEN_SIBLINGS.0 | ListFormat::NO_INTERVENING_COMMENTS.0;
    CASE_OR_DEFAULT_CLAUSE_STATEMENTS = ListFormat::INDENTED.0 | ListFormat::MULTI_LINE.0 | ListFormat::NO_TRAILING_NEW_LINE.0 | ListFormat::OPTIONAL_IF_EMPTY.0;
    HERITAGE_CLAUSE_TYPES = ListFormat::COMMA_DELIMITED.0 | ListFormat::SPACE_BETWEEN_SIBLINGS.0 | ListFormat::SINGLE_LINE.0;
    SOURCE_FILE_STATEMENTS = ListFormat::MULTI_LINE.0 | ListFormat::NO_TRAILING_NEW_LINE.0;
    DECORATORS = ListFormat::MULTI_LINE.0 | ListFormat::OPTIONAL.0 | ListFormat::SPACE_AFTER_LIST.0;
    TYPE_ARGUMENTS = ListFormat::COMMA_DELIMITED.0 | ListFormat::SPACE_BETWEEN_SIBLINGS.0 | ListFormat::SINGLE_LINE.0 | ListFormat::ANGLE_BRACKETS.0 | ListFormat::OPTIONAL.0;
    TYPE_PARAMETERS = ListFormat::COMMA_DELIMITED.0 | ListFormat::SPACE_BETWEEN_SIBLINGS.0 | ListFormat::SINGLE_LINE.0 | ListFormat::ANGLE_BRACKETS.0 | ListFormat::OPTIONAL.0;
    PARAMETERS = ListFormat::COMMA_DELIMITED.0 | ListFormat::SPACE_BETWEEN_SIBLINGS.0 | ListFormat::SINGLE_LINE.0 | ListFormat::PARENTHESIS.0;
    SINGLE_ARROW_PARAMETER = ListFormat::COMMA_DELIMITED.0 | ListFormat::SPACE_BETWEEN_SIBLINGS.0 | ListFormat::SINGLE_LINE.0;
    INDEX_SIGNATURE_PARAMETERS = ListFormat::COMMA_DELIMITED.0 | ListFormat::SPACE_BETWEEN_SIBLINGS.0 | ListFormat::SINGLE_LINE.0 | ListFormat::INDENTED.0 | ListFormat::SQUARE_BRACKETS.0;
    JS_DOC_COMMENT = ListFormat::MULTI_LINE.0 | ListFormat::ASTERISK_DELIMITED.0;
    IMPORT_CLAUSE_ENTRIES = ListFormat::IMPORT_ATTRIBUTES.0; // Deprecated: Use LFImportAttributes
});

// Go: printer/printer.go:6313 getOpeningBracket
pub(crate) fn get_opening_bracket(format: ListFormat) -> &'static str {
    let brackets = format & ListFormat::BRACKETS_MASK;
    if brackets == ListFormat::BRACES {
        "{"
    } else if brackets == ListFormat::PARENTHESIS {
        "("
    } else if brackets == ListFormat::ANGLE_BRACKETS {
        "<"
    } else if brackets == ListFormat::SQUARE_BRACKETS {
        "["
    } else {
        panic!("Unexpected bracket: {:?}", brackets)
    }
}

// Go: printer/printer.go:6328 getClosingBracket
pub(crate) fn get_closing_bracket(format: ListFormat) -> &'static str {
    let brackets = format & ListFormat::BRACKETS_MASK;
    if brackets == ListFormat::BRACES {
        "}"
    } else if brackets == ListFormat::PARENTHESIS {
        ")"
    } else if brackets == ListFormat::ANGLE_BRACKETS {
        ">"
    } else if brackets == ListFormat::SQUARE_BRACKETS {
        "]"
    } else {
        panic!("Unexpected bracket: {:?}", brackets)
    }
}
