//! Driver for the scanner kernel (full tier of the milestone 4 kernels).
//!
//! Sources, copied from ts-rust crates/ts_goport/src with their module
//! paths mirrored:
//!
//! - frontend/scanner/scanner_p1.rs and scanner_p2.rs: the scanner
//!   (the milestone 4 scanner kernel).
//! - frontend/scanner/regexp.rs: regular expression validation.
//! - frontend/scanner/unicode_properties.rs: Unicode property tables.
//! - frontend/scanner/comment_ranges.rs: comment range iteration.
//! - scanner_util.rs: the utilities the scanner uses, extracted item
//!   by item from the 4657-line original (see that file header).
//! - gostring.rs: the Go string port form (bottom half of
//!   scanner_util in ts-rust).
//! - astdata/syntax_kind.rs and astdata/text.rs: token kinds, ranges.
//! - flags.rs and flags_macros.rs: Go enum and flag types.
//! - diagnostics.rs and diag.rs: Message type and the 69 catalog
//!   entries the scanner names (extracted, see those files).
//! - program.rs: CommentDirective, CommentRange and the NodeFactory
//!   constructor the comment ranges use (extracted).
//! - jsnum.rs: stand-in for the two jsnum entry points (see header).
//!
//! External crates are replaced in support.rs: rustc-hash, indexmap
//! and memchr. Edits inside copied files are marked KERNEL-EDIT.
//! scanner_ls (AST-based language service helpers) is not extracted.
//!
//! The driver scans each input file in the Standard and JSX language
//! variants and prints the token stream, the errors the scanner
//! reports, the comment directives and the leading comment ranges of
//! the first token. Input text is converted to the Go port form first,
//! as in ts-rust. Output must be byte-equal between rustc and lust.

#![allow(dead_code, unused_imports, unused_variables, unused_mut, non_snake_case)]

#[path = "flags_macros.rs"]
mod flags_macros_impl;

/// Mirror of the ts-rust lib.rs shim: the go macros are macro_export
/// (crate root), and files import them from crate::flags_macros.
mod flags_macros {
    pub(crate) use crate::{go_enum, go_flags};
}

pub mod astdata;
pub mod core;
pub mod diag;
pub mod diagnostics;
pub mod flags;
pub mod frontend;
pub mod gostring;
pub mod jsnum;
pub mod options;
pub mod program;
pub mod scanner_util;
mod support;

// The kernel sources name the external crates directly. The root
// re-exports bind those names to the local stand-ins in support.rs.
pub use support::{indexmap, memchr, rustc_hash};

/// Kernel mirror of the ts-rust crate prelude: only the names the
/// kernel files use through the glob.
pub mod prelude {
    pub use crate::args;
    pub use crate::astdata::SyntaxKind;
    pub use crate::core::{TextRange, compare_text_ranges};
    pub use crate::diag;
    pub use crate::flags::*;
    pub use crate::indexmap::{IndexMap, IndexSet};
    pub use crate::program::*;
    pub use crate::rustc_hash::{FxHashMap, FxHashSet};
    pub use crate::scanner_util::*;
    pub use crate::unported;
    pub use std::cell::RefCell;
    pub use std::rc::Rc;
}

use std::cell::RefCell;
use std::rc::Rc;

use astdata::SyntaxKind;
use flags::LanguageVariant;
use frontend::scanner::{get_leading_comment_ranges, new_scanner};
use gostring::go_string_from_utf8;
use program::NodeFactory;

/// Escape a token value for one-line output.
fn esc(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c {
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{{{:x}}}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/// Kinds after which a `/` is division, not a regex start. This is the
/// parser rule in simplified form; the driver uses it to decide when to
/// call re_scan_slash_token, as the real parser does.
fn ends_operand(kind: Option<SyntaxKind>) -> bool {
    matches!(
        kind,
        Some(
            SyntaxKind::Identifier
                | SyntaxKind::NumericLiteral
                | SyntaxKind::BigIntLiteral
                | SyntaxKind::StringLiteral
                | SyntaxKind::NoSubstitutionTemplateLiteral
                | SyntaxKind::TemplateTail
                | SyntaxKind::RegularExpressionLiteral
                | SyntaxKind::CloseParenToken
                | SyntaxKind::CloseBracketToken
                | SyntaxKind::CloseBraceToken
                | SyntaxKind::PlusPlusToken
                | SyntaxKind::MinusMinusToken
                | SyntaxKind::ThisKeyword
                | SyntaxKind::TrueKeyword
                | SyntaxKind::FalseKeyword
                | SyntaxKind::NullKeyword
        )
    )
}

/// Scan one text in one language variant and print the token stream.
/// The two parser rescan rules (template middle and tail after a `}`,
/// regex after a non-operand) run here so the template and regexp
/// paths of the scanner execute, as they do under the real parser.
fn scan_pass(text: &str, variant: LanguageVariant, label: &str) {
    println!("-- pass {label}");
    let errors: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let sink = Rc::clone(&errors);
    let mut s = new_scanner();
    s.set_on_error(Some(Box::new(
        move |message, pos, length, args: Vec<String>| {
            let formatted = message
                .format(&args)
                .unwrap_or_else(|e| format!("<format error {e}>"));
            sink.borrow_mut()
                .push(format!("TS{} at {}+{}: {}", message.code(), pos, length, formatted));
        },
    )));
    s.set_language_variant(variant);
    s.set_text(text);
    let mut first_token_pos = 0;
    let mut prev: Option<SyntaxKind> = None;
    // Brace depth of each open template substitution.
    let mut template_stack: Vec<i32> = Vec::new();
    let mut brace_depth: i32 = 0;
    loop {
        let mut kind = s.scan();
        if kind == SyntaxKind::OpenBraceToken {
            brace_depth += 1;
        } else if kind == SyntaxKind::CloseBraceToken {
            if template_stack.last() == Some(&brace_depth) {
                kind = s.re_scan_template_token(false);
                if kind == SyntaxKind::TemplateTail {
                    template_stack.pop();
                }
            } else {
                brace_depth -= 1;
            }
        }
        if kind == SyntaxKind::TemplateHead {
            template_stack.push(brace_depth);
        }
        if (kind == SyntaxKind::SlashToken || kind == SyntaxKind::SlashEqualsToken)
            && !ends_operand(prev)
        {
            kind = s.re_scan_slash_token(true);
        }
        let value = s.token_value();
        println!(
            "tok {:?}({}) {}..{} lb={} flags={:?} text={:?} value={:?}",
            kind,
            kind as u16,
            s.token_start(),
            s.token_end(),
            s.has_preceding_line_break() as u8,
            s.token_flags().bits(),
            esc(s.token_text()),
            esc(value),
        );
        if kind == SyntaxKind::EndOfFile {
            break;
        }
        prev = Some(kind);
        if first_token_pos == 0 {
            first_token_pos = s.token_start();
        }
    }
    for d in s.comment_directives() {
        println!(
            "directive {:?} {}..{}",
            d.kind,
            d.loc.pos(),
            d.loc.end()
        );
    }
    for e in errors.borrow().iter() {
        println!("error {e}");
    }
    // Comment ranges that lead the first real token.
    let f = NodeFactory::new();
    for r in get_leading_comment_ranges(&f, text, first_token_pos) {
        println!(
            "leading-comment {:?} {}..{} trailing-nl={}",
            r.kind,
            r.pos(),
            r.end(),
            r.has_trailing_new_line as u8
        );
    }
}

/// One input file: convert to the Go port form, scan in both variants.
fn run_input(name: &str, raw: &str) {
    let text = go_string_from_utf8(raw.to_string());
    println!("== {name} ({} bytes, port form {} bytes)", raw.len(), text.len());
    scan_pass(&text, LanguageVariant::STANDARD, "standard");
    scan_pass(&text, LanguageVariant::JSX, "jsx");
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        // A built-in smoke input so the binary does useful work bare.
        run_input("smoke", "const x: number = 1 + 2;\n");
        return;
    }
    for path in &args {
        let raw = std::fs::read_to_string(path).expect("read input file");
        let name = std::path::Path::new(path)
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.clone());
        run_input(&name, &raw);
    }
}
