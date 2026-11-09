//! Print packs: the synthetic entries that a JS or d.ts print reads, copied
//! from a checker thread to its twin (`program::send_dts_twin_job`).
//!
//! PORT: not in Go (perf). Go prints on any goroutine, because factory nodes
//! are shared pointers. Here the checker keeps its nodes (a copy, not a
//! move), so no checker state that holds a node can read a freed one. The
//! twin puts each entry at its index: the two threads take chunk numbers
//! from one counter (`share_synthetic_chunks`), so a handle names the same
//! node on both. A read on the twin of an entry that no pack copied panics
//! (`Slot::Absent`, or a missing chunk), so a node that the walk misses
//! stops the run and never prints a wrong value.

use super::*;




thread_local! {
    /// The slots that `export_print_pack` saw, one bit per slot index. It
    /// is kept between walks, with every bit clear.
    // PERF: a hash set cost about a third of the walk on effect.
    static SEEN: Cell<Vec<u64>> = const { Cell::new(Vec::new()) };
    /// The slot and data chunks that `install_print_pack` made on this
    /// thread since the last `release_print_packs`.
    static INSTALLED: RefCell<(Vec<u32>, Vec<u32>)> = const { RefCell::new((Vec::new(), Vec::new())) };
}



#[cfg(test)]
mod tests {
    use super::*;

    // A twin that installs the pack of a tree reads the same tree at the
    // same handles, with the parsed-node aliases resolved. A node that the
    // pack does not hold panics on the twin, and the twin's own new nodes
    // never take a chunk number of the checker.
    #[test]
    fn print_pack_round_trip() {
        std::thread::spawn(|| {
            let f = NodeFactory::new();
            let shared = share_synthetic_chunks();
            let a = f.new_identifier("a");
            let b = f.new_identifier("b");
            let list = f.new_node_list(&[a, b]);
            let block = f.new_block(list, false);
            set_node_parent(a, block);
            let outside = f.new_identifier("outside");
            let pack = export_print_pack(&[block], |_, _| {});
            assert_eq!(pack.slot_count(), 3);

            std::thread::spawn(move || {
                install_twin_synthetic_arena(shared);
                install_print_pack(pack);
                assert_eq!(block.kind(), SyntaxKind::Block);
                assert_eq!(block.statement_list().nodes().to_vec(), vec![a, b]);
                assert_eq!(a.text(), "a");
                assert_eq!(a.parent(), block);
                assert!(std::panic::catch_unwind(|| outside.kind()).is_err());
                let own = NodeFactory::new().new_identifier("own");
                assert!(own != a && own != b && own != block && own != outside);
                assert_eq!(own.text(), "own");

                // A release frees the copies and keeps the twin's own nodes.
                release_print_packs();
                assert!(std::panic::catch_unwind(|| a.kind()).is_err());
                assert_eq!(own.text(), "own");
            })
            .join()
            .expect("twin thread panicked");

            // The checker's next chunks skip the numbers the twin took.
            for i in 0..2 * SLOT_CHUNK {
                let n = f.new_identifier(format!("n{i}"));
                assert_eq!(n.text(), format!("n{i}"));
            }
            assert_eq!(a.text(), "a");
            free_synthetic_nodes();
        })
        .join()
        .expect("test thread panicked");
    }
}
