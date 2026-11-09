//! Port of Go `parser/parser.go` lines 2827 to 4183 (unit U6): type nodes,
//! type members, signatures, parameters, modifiers and the start of the
//! expression parser.
//!
//! Callback rules (contract 4.5): Go `func(p *Parser) T` arguments are passed
//! as `&mut Self::method` or `&mut |p: &mut Parser| ...`. That form works for
//! both `impl FnMut`/`impl FnOnce` generics and `&mut dyn FnMut` parameters.
//!
//! Borrowing: Go evaluates call arguments left to right. The Rust code parses
//! children into locals first, in the same order, and then calls the factory,
//! because `self.factory` cannot stay borrowed while a child parse runs.

use crate::frontend::prelude::*;

// PORT: Go reads `p.diagnostics` directly. The parser and the scanner share
// the diagnostics through `Rc<RefCell<ParseDiagnostics>>` (contract 4.5).
// These two helpers are the only places in this file that touch that store.

// PORT: Go repeats this block in parseImportType and parseImportAttributes:
//
//	if len(p.diagnostics) != 0 {
//		lastDiagnostic := p.diagnostics[len(p.diagnostics)-1]
//		if lastDiagnostic.Code() == diagnostics.X_0_expected.Code() {
//			related := ast.NewDiagnostic(nil, core.NewTextRange(openBracePosition, openBracePosition), diagnostics.The_parser_expected_to_find_a_1_to_match_the_0_token_here, "{", "}")
//			lastDiagnostic.AddRelatedInfo(related)
//		}
//	}
//
// Both copies call this helper. Go changes the diagnostic through its pointer;
// Rust changes the stored diagnostic in place.

