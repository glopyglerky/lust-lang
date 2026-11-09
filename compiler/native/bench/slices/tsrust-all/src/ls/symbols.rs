//! Port of Go `ls/symbols.go`.
//!
//! PORT: Go builds document symbols as `[]*lsproto.DocumentSymbol`, and
//! `mergeExpandos` shares symbol pointers between merge targets and mutates
//! them in place. The generated `lsproto::DocumentSymbol` holds its children
//! by value, so the tree is built with `DocSymbol` (`Rc<RefCell<>>`, the Go
//! pointer) and copied into lsproto values at the end. Go `Children` (a
//! `*[]*DocumentSymbol`) is never shared between two symbols here:
//! `newDocumentSymbol` and `mergeExpandos` always make a new one, and the
//! `target.Children = source.Children` branch of `mergeChildren` only runs
//! for a nil target list, which `newDocumentSymbol` never makes. So the list
//! is an owned `Option<Vec<DocSymbol>>`.

use crate::ls::prelude::*;

use crate::frontend::stringutil_ls;
use crate::gostd::unicode;
use crate::spanmap::Feature;


// Go: ls/symbols.go:65 flattenDocumentSymbols
pub fn flatten_document_symbols(
    doc_symbols: &[lsproto::DocumentSymbol],
    document_uri: &lsproto::DocumentUri,
) -> Vec<lsproto::SymbolInformation> {
    // Flatten the hierarchy
    // PORT: the recursive Go closure `flatten` is a nested fn that takes
    // the captured `result` and `documentURI`.
    fn flatten(
        result: &mut Vec<lsproto::SymbolInformation>,
        document_uri: &lsproto::DocumentUri,
        symbols: &[lsproto::DocumentSymbol],
        container_name: Option<&String>,
    ) {
        for symbol in symbols {
            let info = lsproto::SymbolInformation {
                name: symbol.name.clone(),
                kind: symbol.kind,
                location: lsproto::Location {
                    uri: document_uri.clone(),
                    range: symbol.range,
                },
                container_name: container_name.cloned(),
                tags: symbol.tags.clone(),
                deprecated: symbol.deprecated,
            };
            result.push(info);

            // Recursively flatten children with this symbol as container
            if let Some(children) = &symbol.children {
                if !children.is_empty() {
                    flatten(result, document_uri, children, Some(&symbol.name));
                }
            }
        }
    }
    let mut result: Vec<lsproto::SymbolInformation> = Vec::new();
    flatten(&mut result, document_uri, doc_symbols, None);
    result
}

// PORT: Go `*lsproto.DocumentSymbol` while the tree is built and merged (see
// the module comment).
type DocSymbol = Rc<RefCell<DocumentSymbolNode>>;

// PORT: the fields of `lsproto::DocumentSymbol`, with shared children.
struct DocumentSymbolNode {
    name: String,
    detail: Option<String>,
    kind: lsproto::SymbolKind,
    tags: Option<Vec<lsproto::SymbolTag>>,
    deprecated: Option<bool>,
    range: lsproto::Range,
    selection_range: lsproto::Range,
    children: Option<Vec<DocSymbol>>,
}

// PORT: copies the finished `DocSymbol` tree into lsproto values (the JSON
// writer reads the Go pointers the same way).
fn doc_symbols_to_lsp(symbols: &[DocSymbol]) -> Vec<lsproto::DocumentSymbol> {
    symbols
        .iter()
        .map(|symbol| {
            let s = symbol.borrow();
            lsproto::DocumentSymbol {
                name: s.name.clone(),
                detail: s.detail.clone(),
                kind: s.kind,
                tags: s.tags.clone(),
                deprecated: s.deprecated,
                range: s.range,
                selection_range: s.selection_range,
                children: s.children.as_ref().map(|c| doc_symbols_to_lsp(c)),
            }
        })
        .collect()
}

// Go: ls/symbols.go:94 (the state of the closures in getDocumentSymbolsForChildren)

// PORT: the saved state that the Go `startNode` closure captures for the
// `func()` it returns. `None` is the no-op func for a nil node.
struct EndNode {
    node: Node,
    name: Node,
    save_expando_targets: FxHashSet<String>,
    save_symbols: Vec<DocSymbol>,
}


// Go: ls/symbols.go:282 isPrototypeExpando
// Target is `f.prototype`.

// Go: ls/symbols.go:290 maxLength
const MAX_LENGTH: i32 = 150;


// Go: ls/symbols.go:351 mergeExpandos
// Merges expando symbols into their target symbols, and namespaces of same name.
// Modifies the input slice.
// PORT: Go sets merged entries of the input slice to nil; here the input is
// moved into a `Vec<Option<DocSymbol>>`.

// Go: ls/symbols.go:410 mergeChildren

// Go: ls/symbols.go:424 isAnonymousName
// See `getUnnamedNodeLabel`.
pub fn is_anonymous_name(name: &str) -> bool {
    name == "<function>"
        || name == "<class>"
        || name == "export="
        || name == "default"
        || name == "constructor"
        || name == "()"
        || name == "new()"
        || name == "[]"
        || name.ends_with(") callback")
}

// Go: ls/symbols.go:429 getTextOfName

// Go: ls/symbols.go:445 getUnnamedNodeLabel

// Go: ls/symbols.go:486 getCallExpressionName

// Go: ls/symbols.go:501 getCallExpressionLiteralArgs

// Go: ls/symbols.go:511 cleanCallbackText
pub fn clean_callback_text(text: &str) -> String {
    let mut text = text.to_string();
    let truncated = stringutil_ls::truncate_by_runes(&text, MAX_LENGTH);
    if truncated.len() < text.len() {
        text = truncated + "...";
    }
    // PORT: Go `strings.Map` drops the runes for which the func returns -1.
    text.chars().filter(|&r| !is_line_break(r)).collect()
}

// Go: ls/symbols.go:524 getInteriorModule

// Go: ls/symbols.go:531 getModuleName

// Go: ls/symbols.go:540 DeclarationInfo
#[derive(Clone, Debug)]
pub struct DeclarationInfo {
    pub name: String,
    pub declaration: Node,
    pub match_score: i32,
}

// Go: ls/symbols.go:546 ProvideWorkspaceSymbols
// PORT: Go `[]*compiler.Program` is `&[Rc<compiler::NewProgram>]`, Go
// `*lsconv.Converters` is `&lsconv::Converters`, and the preferences are
// passed by reference.

// Go: ls/symbols.go:615 shouldExcludeFile
// PORT: Go takes the `*ast.SourceFile`; `NewProgram::is_lib_file` takes the
// parsed file, so this does too.

// Go: ls/symbols.go:619 isInsideNodeModules
pub fn is_inside_node_modules(file_name: &str) -> bool {
    file_name.contains("/node_modules/")
}

// Go: ls/symbols.go:628 getMatchScore
// Return a score for matching `s` against `pattern`. In order to match, `s` must contain each of the characters in
// `pattern` in the same order. Upper case characters in `pattern` must match exactly, whereas lower case characters
// in `pattern` match either case in `s`. If `s` doesn't match, -1 is returned. Otherwise, the returned score is the
// number of characters in `s` that weren't matched. Thus, zero represents an exact match, and higher values represent
// increasingly less specific partial matches.
// PORT: Rust strings are valid UTF-8, so Go's `RuneError` for invalid bytes
// does not occur.

/// Go `unicode.IsUpper`: general category Lu.
// PORT: the Rust `Uppercase` property is Lu plus `Other_Uppercase`; the
// `Other_Uppercase` ranges are removed so the result is Go's category test
// (the same helper as in `ls/autoimport/util.rs`).
fn unicode_is_upper(c: char) -> bool {
    if !c.is_uppercase() {
        return false;
    }
    !matches!(
        c as u32,
        0x2160..=0x216F | 0x24B6..=0x24CF | 0x1F130..=0x1F149 | 0x1F150..=0x1F169 | 0x1F170..=0x1F189
    )
}

// Go: ls/symbols.go:649 compareDeclarationInfos
// Sort DeclarationInfos by ascending match score, then ascending case insensitive name, then
// ascending case sensitive name, and finally by source file name and position.

// Go: ls/symbols.go:669 getSymbolKindFromNode
// getSymbolKindFromNode converts an AST node to an LSP SymbolKind.
// Combines getNodeKind with VS Code's fromProtocolScriptElementKind.

#[cfg(test)]
mod match_score_tests {
    use super::get_match_score;

    // Go `getMatchScore` (ls/symbols.go:628) lowers both runes with
    // `unicode.ToLower`. At pin N that is go1.27.1 (Unicode 17.0.0), where
    // U+A7CB lowers to U+0264, so the lower case pattern matches the capital
    // in a string-named declaration. An upper case pattern rune must match
    // exactly.
    #[test]
    fn match_score_uses_unicode_17_case() {
        assert_eq!(get_match_score("\u{A7CB}ab", "\u{264}ab"), 0);
        assert_eq!(get_match_score("x\u{10D50}", "\u{10D70}"), 1);
        assert_eq!(get_match_score("\u{264}ab", "\u{A7CB}"), -1);
    }
}
