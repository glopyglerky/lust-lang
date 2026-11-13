use crate::prelude::*;
use std::cell::OnceCell;
use std::rc::Weak;

// Port of internal/checker/emitresolver.go lines 1 to 600. Lines 601 to 1298
// are in emit_resolver_p2.rs, which adds `impl EmitResolver` blocks.
//
// PORT: layout of the resolver.
// - Go `EmitResolver` holds `*Checker` and `checkerMu`. Here the checkers
//   live in the program pool, so the resolver keeps the pool index of its
//   checker and borrows the checker for each locking call through
//   `with_checker` (Go `checkerMu.Lock()`). A nested lock panics on the
//   pool `RefCell`, as the Go mutex would deadlock.
// - A language-service pool keeps its checkers on the dispatch thread as
//   `Rc<RefCell<Checker>>`, where there is no compile worker checker. A
//   resolver from `get_emit_resolver_of_shared_checker` also keeps a weak
//   link to that `Rc`, and `with_checker` borrows the checker through it.
// - Go unexported methods take `c: &mut Checker` as their first argument
//   (the checker that Go reaches through `r.checker`). Callers that already
//   hold the checker (node builder, symbol accessibility) use them through
//   `c.get_emit_resolver()`.
// - Go exported methods are the `crate::printer::EmitResolver` trait impl
//   below. The trait impl must be one block, so it is in this file. Methods
//   ported in part 2 are inherent methods of `EmitResolver` with the trait
//   signature (Go exported name in snake case; `_exported` suffix when an
//   unexported Go method has the same snake name), and the trait impl calls
//   them.
// - The link stores are `RefCell`s, so the resolver is shared as
//   `Rc<EmitResolver>`. Borrows are never held across a checker call.

// Go: checker/emitresolver.go:20 JSXLinks
// Links for jsx
#[derive(Clone, Copy, Default)]
pub struct JSXLinks {
    pub import_ref: Node,
}

// Go: checker/emitresolver.go:26 DeclarationLinks
// Links for declarations
#[derive(Clone, Copy, Default)]
pub struct DeclarationLinks {
    pub is_visible: Tristate, // if declaration is depended upon by exported declarations
}

// Go: checker/emitresolver.go:30 DeclarationFileLinks
#[derive(Clone, Copy, Default)]
pub struct DeclarationFileLinks {
    pub aliases_marked: bool, // if file has had alias visibility marked
}

// Go: checker/emitresolver.go:34 EmitResolver
// PORT: `checker` and `checkerMu` are `checker_index`, or `shared_checker`
// for a language-service checker (see the file comment).
// PORT: Go `isValueAliasDeclaration` and `aliasMarkingVisitor` are method
// values cached to avoid closure allocation. Here they are plain calls to
// `is_value_alias_declaration_worker` and `alias_marking_visitor_worker`.
// PORT: Go caches referenceResolver binder.ReferenceResolver; part 2 builds one per call in get_reference_resolver (it has no state when all hooks are set).

// Go: checker/emitresolver.go:45 newEmitResolver
// PORT: Go `checker.id` is `checker_index + 1` (see `Checker::new`).


// Go: checker/checker.go:32651 GetEmitResolver, for a checker that a
// language-service pool shares as `Rc<RefCell<Checker>>`.
// PORT: the resolver also links to `checker`, so `with_checker` can reach it
// on the dispatch thread (Go `r.checker`). The checker owns one resolver, so
// the link never changes.


// Go: checker/emitresolver.go:249 isCommonJSModuleExports

/// The nodes that the alias marking walk (`alias_marking_visitor_worker`)
/// must enter in a published, alias-free store with local parents: each
/// node of the store that the walk can act on (an ExportAssignment, an
/// ExportSpecifier, and in a CommonJS file a BinaryExpression that is the
/// expression of a source file statement, see `is_common_js_module_exports`)
/// and its ancestors. One bit per slot.
// PERF: aliaswalk1. Go walks every node of the file. The port did the
// same, and in an edited file each node data read is a scoped read: on an
// effect Option.ts edit the walk took 133 us against 56 us in Go (stable
// build, mini-743d, bpftrace). A scan of the kind column (no node data
// read) finds the nodes that the walk can act on, and their parent chains
// give the paths to them: 17 us.
// The Go parser sets the parent of each child of a node when it finishes
// the node (`overrideParentInImmediateChildren`, `set_parent_in_store_children`
// here), so the parent chain of a node of the tree is the path of
// `ForEachChild` calls that reaches it. The walk then enters the nodes on
// those paths in Go order and leaves out only subtrees with no node to act
// on: it marks the same aliases in the same order. A node that is not in
// the tree (JSDoc, a discarded speculative parse) can add a path that the
// walk never reaches.
struct AliasWalkPaths {
    /// The store id.
    file: usize,
    bits: Vec<u64>,
}



impl AliasWalkPaths {


    /// False when the walk can leave `node` out: a node of the store that is
    /// on no path. A node of another store is entered.
    fn enters(&self, node: Node) -> bool {
        if node.file_index() != self.file {
            return true;
        }
        let index = node.node_id().index();
        self.bits[index / 64] & (1u64 << (index % 64)) != 0
    }
}

// Go: checker/emitresolver.go:311 getMeaningOfEntityNameReference

// Go: checker/emitresolver.go:17 `var _ printer.EmitResolver = (*EmitResolver)(nil)`
// PORT: Go exported methods. Methods from emitresolver.go lines 1 to 600 are
// ported here in full. The others call the part 2 inherent methods.
