//! Port of `checker/links.go` (tsgo#4329).
//!
//! PORT: Go adds `core.PagedLinkStore` (pages of 256 values, a page list for
//! low page indexes and a map for high ones) and the two stores below on top
//! of it. `core::LinkStore` is paged the same way for every store: node keys
//! use per-file page tables of 64-key pages, arena keys (symbols) use one
//! page table, and a page holds the values. So both Go stores map onto
//! `core::LinkStore`; a store of large values holds `Box<V>`, like Go
//! `symbolArenaLinkStore`. `has` and `try_get` keep the exact "a record
//! exists" answer. Go
//! `nodeLinkStore.TryGet` also answers a zero record for a key in an
//! allocated page; its one reader (`tryGetResolvedSymbolFromTypeNode`) then
//! reads a nil `resolvedSymbol`, which is the same result as no record.

use crate::prelude::*;

// Go: checker/links.go:10 nodeLinkStore
/// A links store keyed by node references (Go stores the values in the pages).
pub type NodeLinkStore<V> = LinkStore<Node, V>;

// Go: checker/links.go:28 symbolArenaLinkStore





#[cfg(test)]
mod tests {
    use super::*;

    /// Each read of `ValueSymbolLinkStore` that Go makes gives the symbol
    /// its id once, also when no record exists (`try_get_by_id`,
    /// `has_by_id`: Go `TryGet` and `Has` key by `ast.GetSymbolId`). The
    /// port memo read gives none. A test thread starts with no ids.
    #[test]
    fn each_go_read_gives_the_symbol_its_id_once() {
        let mut symbols = SymbolArena::new();
        let [tried, had, got, inserted, memo] = ["tried", "had", "got", "inserted", "memo"]
            .map(|name| symbols.new_symbol(SymbolFlags::NONE, name));
        let mut store = ValueSymbolLinkStore::default();
        let next = || next_ids().1;
        assert_eq!(next(), 0);

        assert!(store.try_get_by_id(&symbols, tried).is_none());
        assert_eq!(next(), 1, "TryGet gives the id with no record");
        assert!(!store.has_by_id(&symbols, had));
        assert_eq!(next(), 2, "Has gives the id with no record");
        store.get_by_id(&symbols, got);
        store.get_by_id(&symbols, got);
        assert_eq!(next(), 3, "Get gives the id once");
        assert!(store.try_get_by_id(&symbols, got).is_some());
        store.insert_new_by_id(&symbols, inserted, ValueSymbolLinks::default());
        assert_eq!(next(), 4, "a new record gives the id");
        assert!(store.try_get_without_id(memo).is_none());
        assert_eq!(next(), 4, "the memo read gives no id");
        assert_eq!(
            [tried, had, got, inserted].map(|symbol| get_symbol_id(&symbols, symbol)),
            [1, 2, 3, 4]
        );
        assert_eq!(next(), 4);
    }
}
