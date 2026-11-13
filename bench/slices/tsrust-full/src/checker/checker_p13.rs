//! Port of Go `checker/checker.go` lines 11172-12100.
//!
//! PORT: Go `c.error(...)` returns the `*ast.Diagnostic` it already added, and
//! callers then mutate it (`AddRelatedInfo`). Diagnostics are owned values
//! here, so those call sites build the diagnostic with
//! `new_diagnostic_for_node`, add the related information, then call
//! `self.add_diagnostic` (the same two steps Go `error` does). The collection
//! is append-only, so the stored result is the same.

use crate::prelude::*;


// Go: checker/checker.go:12215 getThisParameterFromNodeContext


/// Slots of `Checker::control_flow_containers` (16 KiB). On T3 Code, the
/// gate projects and 5 realworld4 repos, the misses of 1024 slots are at most
/// 2 points more than those of a memo with no limit (effect: 38% against
/// 23%). 256 slots cost effect 0.07% more instructions; a `LinkStore` memo
/// costs more instructions than this table everywhere except effect.
pub const CONTROL_FLOW_CONTAINER_SLOTS: usize = 1024;

/// The slot of `node` in `Checker::control_flow_containers`: the high bits
/// of a Fibonacci hash of the handle (file and slot index).
#[inline]
fn control_flow_container_slot(node: Node) -> usize {
    const BITS: u32 = CONTROL_FLOW_CONTAINER_SLOTS.trailing_zeros();
    (node.0.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> (64 - BITS)) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    // Go `getSymbolNameForPrivateIdentifier` (binder/binder.go) writes the
    // class symbol's id into the name of each private identifier symbol.
    // Each file binds into its own arena here, and
    // `SymbolArena::prepare_file_arena` moves the ids, and so these names,
    // when the arena joins the program after the lib files. The members and
    // exports tables must then find the moved names: their hash filter and
    // index are built again (corefix1). The checker finds each private name
    // as Go `lookupSymbolForPrivateIdentifierDeclaration` (checker.go:11674)
    // does, in tables of 2, 8, 9 and 41 entries (with no index and with
    // one).
    #[test]
    fn private_names_are_found_after_the_file_arena_moves() {
        let mut source = String::new();
        for (class, size) in [1, 7, 8, 40].into_iter().enumerate() {
            source.push_str(&format!("export class C{class} {{\n"));
            for k in 0..size {
                source.push_str(&format!("    #p{k} = {k};\n    static #s{k} = {k};\n"));
            }
            source.push_str("    m() {}\n}\n");
        }
        let dir =
            std::env::temp_dir().join(format!("ts_goport_private_names_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.ts"), source).unwrap();
        std::fs::write(
            dir.join("tsconfig.json"),
            r#"{ "compilerOptions": { "target": "es2022", "types": [] }, "files": ["a.ts"] }"#,
        )
        .unwrap();
        let config = dir.join("tsconfig.json");
        let program = crate::program::try_load_version(&config.to_string_lossy(), |_| {})
            .unwrap_or_else(|e| panic!("cannot load {}: {e}", config.display()));
        let _ = std::fs::remove_dir_all(&dir);
        let scope = crate::core::enter_program(Some(program));
        let file = program
            .source_files()
            .find(|file| file.info.file_name.ends_with("/a.ts"))
            .expect("a.ts is not in the program")
            .root;
        let (found, missing) = crate::program::with_type_checker_for_file(file, move |checker| {
            let (mut found, mut missing) = (0, Vec::new());
            for class in file.statements().iter() {
                let method = class.members().iter().last().expect("m");
                for member in class.members().iter() {
                    let name = member.name();
                    if !is_private_identifier(name) {
                        continue;
                    }
                    let symbol = checker
                        .lookup_symbol_for_private_identifier_declaration(name.text(), method);
                    if symbol.is_some() && symbol == checker.get_symbol_of_declaration(member) {
                        found += 1;
                    } else {
                        missing.push(name.text().to_string());
                    }
                }
            }
            (found, missing)
        });
        drop(scope);
        crate::program::release_program(program);
        assert!(missing.is_empty(), "not found: {missing:?}");
        assert_eq!(found, 2 * (1 + 7 + 8 + 40));
    }

    /// Go `getControlFlowContainer` (checker.go:11638) as written: a walk of
    /// `node.Parent` and its parents with no memo.
    fn go_control_flow_container(node: Node) -> Node {
        let mut n = node.parent();
        while n.is_some() {
            if is_function_like(n) && get_immediately_invoked_function_expression(n).is_nil()
                || is_module_block(n)
                || is_source_file(n)
                || is_property_declaration(n)
            {
                return n;
            }
            n = n.parent();
        }
        Node::NIL
    }

    // cfcache1: `get_control_flow_container` keeps answers in a small table.
    // On every node of a TS file, a JS file and lib.es5.d.ts, in tree order
    // and then in reverse (so that slots are taken again by other nodes),
    // each answer must be the walk's answer, from the table or not.
    #[test]
    fn control_flow_container_memo_gives_the_walk_answer() {
        let a = r#"
namespace N { export const x = 1; function f() { return x; } namespace M { export let y = x; } }
declare module "m" { export const z: number; }
class C {
    p = 1;
    q = () => this.p;
    static s = (function () { return 1; })();
    static { const b = C.s; }
    constructor(public w: number) { const v = ((() => w))(); }
    m(this: C) { const v = [1].map(a => a + this.p); return v; }
    get g() { return 1; }
    set g(v: number) { this.p = v; }
}
const iife = (() => { const inner = 1; return inner; })();
const iife2 = (function named() { return 2; }());
const iife3 = ((async () => 3))();
const obj = { method() { return 1; }, arrow: () => 2, get acc() { return 3; }, nested: { deep() { return () => 4; } } };
function outer(a: number) {
    function inner() { return a; }
    label: for (const k of [1]) { if (k) { break label; } }
    return [1, 2].map(function (b) { return b + a + inner(); });
}
function* gen() { yield 1; }
enum E { A = 1, B = A + 1 }
type T<U> = U extends string ? { [K in keyof U]: U[K] } : never;
interface I { (x: number): string; new (y: string): I; method?(): void; prop: (z: number) => void; }
const cls = class { field = (() => 1)(); m() { return this.field; } };
export default class { field = 1; }
"#;
        let b = r#"
/** @param {number} a @returns {number} */
function j(a) { return a + 1; }
/** @type {(x: number) => number} */
const k = (x) => x;
module.exports.k = k;
(function () { var hidden = 1; return hidden; })();
"#;
        let dir = std::env::temp_dir().join(format!("ts_goport_cfcache_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.ts"), a).unwrap();
        std::fs::write(dir.join("b.js"), b).unwrap();
        std::fs::write(
            dir.join("tsconfig.json"),
            r#"{ "compilerOptions": { "target": "es2022", "types": [], "allowJs": true, "checkJs": true, "noEmit": true }, "files": ["a.ts", "b.js"] }"#,
        )
        .unwrap();
        let config = dir.join("tsconfig.json");
        let program = crate::program::try_load_version(&config.to_string_lossy(), |_| {})
            .unwrap_or_else(|e| panic!("cannot load {}: {e}", config.display()));
        let _ = std::fs::remove_dir_all(&dir);
        let scope = crate::core::enter_program(Some(program));
        let roots: Vec<Node> = program
            .source_files()
            .filter(|file| {
                let name = &file.info.file_name;
                name.ends_with("/a.ts")
                    || name.ends_with("/b.js")
                    || name.ends_with("/lib.es5.d.ts")
            })
            .map(|file| file.root)
            .collect();
        assert_eq!(roots.len(), 3, "a.ts, b.js and lib.es5.d.ts");
        let mut nodes = Vec::new();
        fn collect(node: Node, nodes: &mut Vec<Node>) {
            nodes.push(node);
            node.for_each_child(|child| {
                collect(child, nodes);
                false
            });
        }
        for &root in &roots {
            collect(root, &mut nodes);
        }
        let expected: Vec<Node> = nodes
            .iter()
            .map(|&n| go_control_flow_container(n))
            .collect();
        let total = nodes.len();
        let (wrong, kept) = crate::program::with_type_checker_for_file(roots[0], move |checker| {
            let (mut wrong, mut kept) = (Vec::new(), 0usize);
            let mut check = |checker: &mut Checker, i: usize| {
                let got = checker.get_control_flow_container(nodes[i]);
                if got != expected[i] {
                    wrong.push((nodes[i], got, expected[i]));
                }
                if checker.control_flow_containers[control_flow_container_slot(nodes[i])]
                    == (nodes[i], expected[i])
                {
                    kept += 1;
                }
            };
            for i in 0..nodes.len() {
                check(checker, i);
                check(checker, i);
            }
            for i in (0..nodes.len()).rev() {
                check(checker, i);
            }
            (wrong, kept)
        });
        drop(scope);
        crate::program::release_program(program);
        assert!(
            wrong.is_empty(),
            "{} wrong answers, first: {:?}",
            wrong.len(),
            wrong.first()
        );
        // More nodes than slots, so slots are taken again. Only the 3 roots
        // (no parent) are not kept: 10,673 nodes, 32,010 of 32,019 checks.
        assert!(total > 4 * CONTROL_FLOW_CONTAINER_SLOTS, "{total} nodes");
        assert!(
            kept * 10 >= total * 3 * 9,
            "the table kept {kept} of {} answers",
            total * 3
        );
    }

    // cfcache1 (R183 reviewer item 4): the table keeps an answer only from
    // a store whose parents are all in that store (`parents_local`). The
    // test of a function-like node reads its parent, which can be outside
    // the store when some parent of the store is. Stores `a` and `c` each
    // hold `x;` in a module block; in `a` the parent of that block is a
    // node of store `b`. Both walks end at the block, but only the answer
    // from `c` is kept. It publishes, so no other test may build or publish
    // stores while it runs (the runner uses one thread).
    #[test]
    fn control_flow_container_memo_keeps_only_stores_with_local_parents() {
        let a = new_file_store("/cfcache_local/a.ts", "x;");
        let b = new_file_store("/cfcache_local/b.ts", "y;");
        let c = new_file_store("/cfcache_local/c.ts", "x;");
        let module_block_of_x = |file: usize| {
            let f = NodeFactory::for_file(file);
            let x = f.new_identifier("x");
            let statement = f.new_expression_statement(x);
            let block = f.new_module_block(f.new_node_list(&[statement]));
            set_node_parent(x, statement);
            set_node_parent(statement, block);
            (x, block)
        };
        let (x_a, block_a) = module_block_of_x(a);
        let (x_c, block_c) = module_block_of_x(c);
        let y = NodeFactory::for_file(b).new_identifier("y");
        set_node_parent(block_a, y);
        for file in [a, b, c] {
            freeze_file_store(file);
        }
        crate::program::publish_parsed_files("/");
        let parents_local = |file| frozen_store_facts(file).map(|facts| facts.parents_local);
        assert_eq!(parents_local(a), Some(false));
        assert_eq!(parents_local(c), Some(true));
        assert_eq!(go_control_flow_container(x_a), block_a);
        assert_eq!(go_control_flow_container(x_c), block_c);

        let dir =
            std::env::temp_dir().join(format!("ts_goport_cfcache_local_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("main.ts"), "export {};\n").unwrap();
        std::fs::write(
            dir.join("tsconfig.json"),
            r#"{ "compilerOptions": { "types": [], "noEmit": true }, "files": ["main.ts"] }"#,
        )
        .unwrap();
        let config = dir.join("tsconfig.json");
        let program = crate::program::try_load_version(&config.to_string_lossy(), |_| {})
            .unwrap_or_else(|e| panic!("cannot load {}: {e}", config.display()));
        let _ = std::fs::remove_dir_all(&dir);
        let scope = crate::core::enter_program(Some(program));
        let root = program
            .source_files()
            .find(|file| file.info.file_name.ends_with("/main.ts"))
            .expect("main.ts is not in the program")
            .root;
        let answers = crate::program::with_type_checker_for_file(root, move |checker| {
            [x_a, x_c].map(|x| {
                let got = checker.get_control_flow_container(x);
                let kept =
                    checker.control_flow_containers[control_flow_container_slot(x)] == (x, got);
                (got, kept)
            })
        });
        drop(scope);
        crate::program::release_program(program);
        assert_eq!(answers, [(block_a, false), (block_c, true)]);
    }
}
