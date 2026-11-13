//! Kernel extraction of the one type the scanner uses from ts-rust
//! program.rs, copied verbatim.

use crate::astdata::SyntaxKind;
use crate::core::TextRange;
use crate::flags::CommentDirectiveKind;

/// Go `ast.CommentRange` (copied from printer/printer_p1.rs, where the
/// port defines it until an ast module lands).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommentRange {
    pub text_range: TextRange,
    pub kind: SyntaxKind,
    pub has_trailing_new_line: bool,
}

impl CommentRange {
    pub fn pos(&self) -> i32 {
        self.text_range.pos()
    }

    pub fn end(&self) -> i32 {
        self.text_range.end()
    }

    pub fn len(&self) -> i32 {
        self.text_range.len()
    }
}

/// Stand-in for the ts-rust NodeFactory: the scanner only calls
/// new_comment_range on it, which is a pure constructor (copied
/// verbatim from frontend/ast_factory_p2.rs). The real factory builds
/// arena nodes; no scanner file in this kernel needs one.
#[derive(Clone, Copy, Debug, Default)]
pub struct NodeFactory;

impl NodeFactory {
    pub fn new() -> NodeFactory {
        NodeFactory
    }

    // Go: ast/ast.go:3115 NewCommentRange
    #[must_use]
    pub fn new_comment_range(
        &self,
        kind: SyntaxKind,
        pos: i32,
        end: i32,
        has_trailing_new_line: bool,
    ) -> CommentRange {
        CommentRange {
            text_range: TextRange::new(pos, end),
            kind,
            has_trailing_new_line,
        }
    }
}

/// Go `ast.CommentDirective`.
#[derive(Clone, Copy, Debug, Default)]
pub struct CommentDirective {
    pub loc: TextRange,
    pub kind: CommentDirectiveKind,
}
