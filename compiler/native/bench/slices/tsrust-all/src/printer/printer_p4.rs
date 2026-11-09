use crate::gostd::debug::kind_string;
use crate::prelude::*;

// Port of printer/printer.go lines 3690 to 4691: declarations, module
// references, JSX, clauses, property assignments, enum members, JSDoc and
// top-level nodes.
//
// PORT: every Go `*ast.X` parameter is a `Node`. Go `node.AsX()` conversions
// are dropped. Go `(*Printer).emitX` method values passed to `emitList` are
// `Printer::emit_x` (`fn(&mut Printer, Node)`).
// PORT: Go variadic `greatestEnd(end, nodes...)` is
// `greatest_end(end, &[&a, &b])`, where each element is a `Node`,
// `NodeList`, `ModifierList` or `TextRange` behind the `tryGetEnd` trait of
// printer/utilities.rs.

//
// Declarations
//


//
// Module references
//


//
// JSX
//


//
// Clauses
//


//
// Property assignments
//


//
// Enum
//


//
// JSDoc
//


//
// Top-level nodes
//


#[cfg(test)]
mod tests {
    use super::*;
    use std::panic::AssertUnwindSafe;

    // Go prints `Kind.String()` in an unhandled-kind panic
    // (printer/printer.go:4233 `panic(fmt.Sprintf("unhandled statement: %v", node.Kind))`).
    // The API's printNode answers this text for a reparsed `@import` (apifuzz2 N1).
    #[test]
    fn unhandled_statement_panic_names_the_go_kind() {
        let f = NodeFactory::new();
        let specifier = f.new_string_literal("y", TokenFlags::NONE);
        let decl = f.new_js_import_declaration(ModifierList::NIL, Node::NIL, specifier, Node::NIL);
        let mut printer = new_printer(PrinterOptions::default(), PrintHandlers::default(), None);
        let payload = std::panic::catch_unwind(AssertUnwindSafe(|| printer.emit(decl, Node::NIL)))
            .expect_err("no panic");
        assert_eq!(
            payload.downcast_ref::<String>().map(String::as_str),
            Some("unhandled statement: KindJSImportDeclaration")
        );
    }

    // Go prints `%v` of the precedence (printer/printer.go:2867
    // `panic(fmt.Sprintf("unhandled precedence: %v", precedence))`).
    // `ast.OperatorPrecedence` is an int with no `String` method, so Go prints
    // the number. A binary expression whose operator is not a binary operator
    // has the invalid precedence -1 (gaps2a skeptic repro r-prec).
    #[test]
    fn unhandled_precedence_panic_prints_the_number() {
        let f = NodeFactory::new();
        let (left, right) = (f.new_identifier("x"), f.new_identifier("y"));
        let operator = f.new_token(SyntaxKind::OpenBraceToken);
        let binary = f.new_binary_expression(ModifierList::NIL, left, Node::NIL, operator, right);
        let mut printer = new_printer(PrinterOptions::default(), PrintHandlers::default(), None);
        let payload = std::panic::catch_unwind(AssertUnwindSafe(|| {
            printer.get_binary_expression_precedence(binary)
        }))
        .expect_err("no panic");
        assert_eq!(
            payload.downcast_ref::<String>().map(String::as_str),
            Some("unhandled precedence: -1")
        );
    }
}
