use crate::ls::autoimport::prelude::*;

// Port of Go `ls/autoimport/import_adder.go`.
//
// PORT (whole file):
// - Pinned decision (map-ls-completions 2.5): Go `importAdder` stores
//   `checker *checker.Checker`. The Rust adder does not store the checker;
//   the methods that use it take `&mut Checker`, and `NewImportAdder` drops
//   the `checker` parameter. Functions that reach those methods
//   (`TypeNodeToAutoImportableTypeNode`, `importSymbols`) also take it.
// - The Go interface is the trait `ImportAdder`; the Go struct
//   `importAdder` is `ImportAdderImpl` (Rust type names cannot differ only
//   in case).
// - Go `map[*ast.ImportClauseOrBindingPattern]*addToExistingState` and
//   `map[string]*importsCollection` are `IndexMap`s in insertion order.
//   PORT: Go map order is random. The change tracker sorts edits by
//   position and `insertImports` sorts new declarations, so only ties can
//   come out in another order.
// - Go `*importsCollection` values are shared between the map and the
//   caller of `getNewImportEntry`: `Rc<RefCell<ImportsCollection>>`.
// - Go `symbol.Name` / `symbol.Parent` reads take the symbol arena, as the
//   ast helpers do (`symbols: &SymbolArena`).

use crate::frontend::compiler;
use crate::gostd::Context;
use crate::locale;
use crate::ls::{change, lsconv, lsutil};
use crate::lsp::lsproto;

// Go: ls/autoimport/import_adder.go:24 ImportAdder

// Go: ls/autoimport/import_adder.go:32 addToExistingState
// addToExistingState tracks modifications to an existing import clause or binding pattern
// PORT: Go always makes `namedImports`, so it is a plain map; Go reads it
// only through `sortedNamedImports`, which sorts the keys.
#[derive(Clone, Debug, Default)]
pub struct AddToExistingState {
    pub import_clause_or_binding_pattern: Node,
    pub default_import: Option<NewImportBinding>,
    pub named_imports: FxHashMap<String, NewImportBinding>,
}

// Go: ls/autoimport/import_adder.go:39 importsCollection
// importsCollection tracks new imports to be created for a given module specifier
// PORT: Go `namedImports` starts nil and is made on first use: `None`.
#[derive(Clone, Debug, Default)]
pub struct ImportsCollection {
    pub default_import: Option<NewImportBinding>,
    pub named_imports: Option<FxHashMap<String, NewImportBinding>>,
    pub namespace_like_import: Option<NewImportBinding>,
    pub use_require: bool,
}

// Go: ls/autoimport/import_adder.go:46 newImportsKey
fn new_imports_key(module_specifier: &str, top_level_type_only: bool) -> String {
    if top_level_type_only {
        return format!("1|{module_specifier}");
    }
    format!("0|{module_specifier}")
}

// Go: ls/autoimport/import_adder.go:53 importAdder
// PORT: see the file header (no `checker` field; Go `*View` is `Rc<View>`,
// Go `*lsconv.Converters` is `Rc<lsconv::Converters>`).

// Go: ls/autoimport/import_adder.go:70 NewImportAdder
// PORT: the Go `checker` parameter is dropped (pinned decision, see the
// file header). Go does not read `program` or `file` either; they stay so
// callers pass the Go arguments.


// Go: ls/autoimport/import_adder.go:178 sortedNamedImports
// PORT: a Go nil map is `None`. Go `slices.Sorted` on string keys is byte
// order, as `String` ordering.
fn sorted_named_imports(m: Option<&FxHashMap<String, NewImportBinding>>) -> Vec<NewImportBinding> {
    let Some(m) = m else {
        return Vec::new();
    };
    let mut keys: Vec<&String> = m.keys().collect();
    keys.sort();
    let mut result: Vec<NewImportBinding> = Vec::with_capacity(keys.len());
    for k in keys {
        result.push(m[k].clone());
    }
    result
}

// Go: ls/autoimport/import_adder.go:330 reduceAddAsTypeOnlyValues
// `NotAllowed` overrides `Required` because one addition of a new import might be required to be type-only
// because of `--importsNotUsedAsValues=error`, but if a second addition of the same import is `NotAllowed`
// to be type-only, the reason the first one was `Required` - the unused runtime dependency - is now moot.
// Alternatively, if one addition is `Required` because it has no value meaning under `--preserveValueImports`
// and `--isolatedModules`, it should be impossible for another addition to be `NotAllowed` since that would
// mean a type is being referenced in a value location.
fn reduce_add_as_type_only_values(
    prev_value: lsproto::AddAsTypeOnly,
    new_value: lsproto::AddAsTypeOnly,
) -> lsproto::AddAsTypeOnly {
    if new_value > prev_value {
        return new_value;
    }
    prev_value
}


// Go: ls/autoimport/import_adder.go:382 TypeToAutoImportableTypeNode
// PORT: Go shares `idToSymbol` with the node builder through
// `c.TypeToTypeNode`. The port builds the node builder as
// `Checker::type_to_type_node_exported` does and reads the filled map back
// from `nb.impl_.borrow().id_to_symbol` (PORTING "Programs and checkers").

// Go: ls/autoimport/import_adder.go:398 TypeNodeToAutoImportableTypeNode
// TypeNodeToAutoImportableTypeNode converts import type references in a type node to
// simple type references and registers needed imports with the import adder.
// PORT: `c` is added first (the adder's `AddImportFromExportedSymbol` takes
// the checker, pinned decision); Go `importAdder` may be nil (`None`).

// Go: ls/autoimport/import_adder.go:415 importSymbols

// Go: ls/autoimport/import_adder.go:427 TryGetAutoImportableReferenceFromTypeNode
// Given a type node containing 'import("./a").SomeType<import("./b").OtherType<...>>',
// returns an equivalent type reference node with any nested ImportTypeNodes also replaced
// with type references, and a list of symbols that must be imported to use the type reference.
// TryGetAutoImportableReferenceFromTypeNode converts import type references in a type node
// to simple type references and returns the transformed type node and the symbols that need
// to be imported.
// PORT: `symbols` is the arena that owns the `idToSymbol` symbols (a
// checker's `symbols`), read by `getNameForExportedSymbol`. The Go closure
// appends to the local `symbols` slice; here that list is the visitor's
// `ctx`. Go `map[*ast.IdentifierNode]*ast.Symbol` is `FxHashMap<Node, SymbolId>`.

// Go: ls/autoimport/import_adder.go:465 getNameForExportedSymbol
// If a type checker and multiple files are available, consider using `forEachNameOfDefaultExport`
// instead, which searches for names of re-exported defaults/namespaces in target files.

// Go: ls/autoimport/import_adder.go:481 replaceFirstIdentifierOfEntityName

