//! Port of `internal/testutil/tsbaseline/type_symbol_baseline.go`: the
//! harness `.types` and `.symbols` writers.
//!
//! PORT: the `testing` wrappers (`DoTypeAndSymbolBaseline`, `checkBaselines`,
//! `isTypeBaselineNodeReuseLine` and the diff fixup) are left out. Callers
//! use `generate_baseline` directly, like the Go dumper `cmd/typesymdump`.

use std::any::Any;
use std::fmt::Write as _;
use std::panic::{AssertUnwindSafe, catch_unwind};

use crate::baseline::util::{
    remove_line_delimiters, 
};
use crate::prelude::*;

/// Go `baseline.NoContent`.
pub const NO_CONTENT: &str = "<no content>";

/// Go `harnessutil.TestFile`.
#[derive(Clone, Debug)]
pub struct TestFile {
    pub unit_name: String,
    pub content: String,
}

/// Go `codeLinesRegexp.Split(s, -1)` with `[\r  ]|\r?\n`. RE2 is
/// leftmost-first, so at `\r\n` the first branch takes `\r` alone and the
/// `\n` splits again: every single `\r`, `\n`, U+2028 and U+2029 separates.
pub fn split_code_lines(text: &str) -> Vec<&str> {
    text.split(['\r', '\n', '\u{2028}', '\u{2029}']).collect()
}



// Go: type_symbol_baseline.go:144 generateBaseline

// Go: type_symbol_baseline.go:196 iterateBaseline

// Go: type_symbol_baseline.go:263 typeWriterWalker
// PORT: `program` is the thread-local loaded program, so it is not a field.
// `declarationTextCache` is only read in this pin (never written), so it is
// left out; its lookup always misses.
pub struct TypeWriterWalker {
    pub had_error_baseline: bool,
    pub current_source_file: Node,
    /// Rust-only: when true, a panic in one node's checker work becomes a
    /// `<<goport panic: MESSAGE>>` result instead of unwinding. The Go
    /// harness has no such guard (a Go panic fails the test).
    pub catch_panics: bool,
    /// Rust-only: the number of nodes that panicked under `catch_panics`.
    pub panic_count: usize,
}

// Go: type_symbol_baseline.go:270 newTypeWriterWalker
pub fn new_type_writer_walker(had_error_baseline: bool) -> TypeWriterWalker {
    TypeWriterWalker {
        had_error_baseline,
        current_source_file: Node::NIL,
        catch_panics: false,
        panic_count: 0,
    }
}

// Go: type_symbol_baseline.go:284 typeWriterResult
#[derive(Clone, Debug, Default)]
pub struct TypeWriterResult {
    pub line: i32,
    pub source_text: String,
    pub symbol: String,
    pub typ: String,
    pub underline: String, // !!!
}

const UNPORTED_PREFIX: &str = "unported Go code";

fn payload_message(payload: &(dyn Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_string()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        String::new()
    }
}

impl TypeWriterWalker {
    // Go: type_symbol_baseline.go:278 getTypeCheckerForCurrentFile
    // PORT: Go returns the checker and a release func. The Rust pool lends
    // the same checker (`GetTypeCheckerForFile`) to a closure on the
    // checker's own thread, so the closure and its result must be `Send`.

    // Go: type_symbol_baseline.go:292 getTypes

    // Go: type_symbol_baseline.go:298 getSymbols

    // Go: type_symbol_baseline.go:304 visitNode


    // Go: type_symbol_baseline.go:346 writeTypeOrSymbol
}


// Go: type_symbol_baseline.go:319 forEachASTNode

// Go: type_symbol_baseline.go:459 isImportStatementName

// Go: type_symbol_baseline.go:472 isExportStatementName

// Go: type_symbol_baseline.go:482 isIntrinsicJsxTag

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_matchers() {
        // CRLF gives an extra empty line (RE2 leftmost-first takes `\r` alone).
        assert_eq!(
            split_code_lines("a\r\nb\nc\u{2028}d"),
            vec!["a", "", "b", "c", "d"]
        );
        assert_eq!(split_code_lines(""), vec![""]);
        // A lone `\r` stays in the line text.
        assert_eq!(remove_line_delimiters("x\r\ny\rz\n"), "xy\rz");
        assert!(is_bracket_line(" \t{ "));
        assert!(is_bracket_line("|"));
        assert!(!is_bracket_line("{}"));
        assert!(!is_bracket_line("\u{A0}}"));
        assert!(!is_bracket_line(""));
    }
}
