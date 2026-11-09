use crate::ls::autoimport::prelude::*;

// Port of Go `ls/autoimport/view.go`.
//
// PORT (whole file):
// - Go `*View` is shared (`Rc<View>` in the import adder) and fills three
//   caches lazily through the pointer. The caches use `RefCell` / `Cell`, so
//   every method takes `&self`.
// - Go passes `v.program` where a `modulespecifiers.ModuleSpecifierGenerationHost`
//   is needed. The host for the installed program is
//   `modulespecifiers::ProgramHost` (PORT: installed program, plan D-LS1).
// - Go `*ast.SourceFile` passed as an `ast.HasFileName` to a program method
//   is `source_file_has_file_name(file)`.

use crate::flags_macros::go_enum;
use crate::frontend::compiler;
use crate::frontend::core_ls_ext::{first_non_zero, min_all_func};
use crate::frontend::core_nodemodules::UNPREFIXED_NODE_CORE_MODULES;
use crate::frontend::module;
use crate::frontend::tspath;
use crate::ls::lsutil;
use crate::lsp::lsproto;
use crate::modulespecifiers;
use std::cell::Cell;

// Go: ls/autoimport/view.go:21 View
// PORT: Go `checker *checker.Checker` (ts#64178) is not a field; see
// `new_view`.
// PORT: Go `*collections.Set[string]` `conditions` is never nil after
// `NewView`, so it is a plain set. Go `*collections.MultiMap` is
// `IndexMap<K, Vec<V>>` behind an `Rc` (Go returns the pointer). The lazy
// Go fields (`allowedEndings`, `existingImports`, `shouldUseRequireForFixes`)
// are `RefCell<Option<..>>` / `Cell<Option<bool>>` (`None` is Go nil).


// Go: ls/autoimport/view.go:37 NewView
// PORT: Go takes `typeChecker` and stores it in the view (ts#64178). The
// Rust checker is a `RefCell` that the caller already borrows, so the view
// does not keep it: the methods that read Go `v.checker` take `ch` (see
// `fix.rs`).


// Go: ls/autoimport/view.go:73 QueryKind
go_enum!(QueryKind, i32 {
    WORD_PREFIX = 0; // QueryKindWordPrefix
    EXACT_MATCH = 1; // QueryKindExactMatch
    CASE_INSENSITIVE_MATCH = 2; // QueryKindCaseInsensitiveMatch
});


// Go: ls/autoimport/view.go:175 FixAndExport
#[derive(Clone, Debug)]
pub struct FixAndExport {
    pub fix: Rc<Fix>,
    pub export: Rc<Export>,
}

/// Go `unicode.IsUpper`: general category Lu.
// PORT: the Rust `Uppercase` property is Lu plus `Other_Uppercase`; the
// `Other_Uppercase` ranges are removed so the result is Go's category test
// (the same helper as in `util.rs`, which keeps it private).
fn unicode_is_upper(c: char) -> bool {
    if !c.is_uppercase() {
        return false;
    }
    !matches!(
        c as u32,
        0x2160..=0x216F | 0x24B6..=0x24CF | 0x1F130..=0x1F149 | 0x1F150..=0x1F169 | 0x1F170..=0x1F189
    )
}

