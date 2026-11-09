use crate::ls::lsutil::prelude::*;

use crate::frontend::core_nodemodules::{
    EXCLUSIVELY_PREFIXED_NODE_CORE_MODULES, 
};
use crate::gostd::unicode;

// Port of Go `ls/lsutil/utilities.go`.

// Go: ls/lsutil/utilities.go:15 ProbablyUsesSemicolons
// PORT: Go `visit` is a recursive closure over the counters. Here it is a
// nested fn that takes them by reference.

// Go: ls/lsutil/utilities.go:77 ShouldUseUriStyleNodeCoreModules

// Go: ls/lsutil/utilities.go:91 QuotePreferenceFromString

// Go: ls/lsutil/utilities.go:98 GetQuotePreference

// Go: ls/lsutil/utilities.go:115 ModuleSymbolToValidIdentifier
// PORT: reading `moduleSymbol.Name` needs the symbol arena, so it is the
// first parameter (as for Go `ast` functions that take a symbol).

// Go: ls/lsutil/utilities.go:123 ModuleSpecifierToValidIdentifier
// PORT: Go `[]rune(s)` turns invalid UTF-8 into U+FFFD; a Rust `&str` is
// always valid, so `chars()` gives the same runes.

// Go: ls/lsutil/utilities.go:158 IsNonContextualKeyword
pub fn is_non_contextual_keyword(token: SyntaxKind) -> bool {
    is_keyword_kind(token) && !is_contextual_keyword(token)
}
