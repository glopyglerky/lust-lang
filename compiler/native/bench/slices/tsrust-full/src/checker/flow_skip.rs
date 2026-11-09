//! flowskip1: skip a flow walk that can only return the declared type and
//! makes nothing on its way (Bun `is_never_narrowed`,
//! `sema/check/flow.rs:4766-4886`, made exact against Go's walk).
//!
//! The Go model is in `studies/flowskip1/go-model.md` with the fixes in
//! `studies/flowskip1/go-model-fix.md` (under
//! `target/continuation-r97-goport`). In short: when every flow node that
//! Go's walk visits is inert for the reference R, `getTypeAtFlowNode`
//! (flow.go:117) visits a fixed path pi(u) from the flow node u of R:
//! - assignments, calls and array mutations continue at their antecedent;
//! - conditions and switch clauses nest on their antecedent;
//! - a branch label nests on its first walked antecedent (the first empty
//!   switch clause is the bypass, flow.go:1260), and the declared type
//!   returns at once there because declared == initial (flow.go:1269);
//! - a Start node moves to the containing function when Go's rule holds
//!   (flow.go:189), else gives the initial type, which is the declared type.
//!
//! On such a path Go makes no type, symbol, signature or diagnostic, writes
//! no lasting state and returns the declared type. A node is inert for R
//! when no role of its expression can match R (mention families), every
//! identifier that Go resolves there for R's class is resolved already, a
//! call has a cached "none" effects signature (flow.go:2047), and the node
//! has no key name resolution (flow.go:1753) and no union-making `&&`/`||`
//! (flow.go:539-551).
//!
//! Index (per checker, per static file): the forest of "next" pointers of
//! pi, an Euler tour of it, and per mention key the sorted entry and exit
//! numbers of the nodes that mention it, so a count of mentions on a path
//! is two binary searches. Per class of reference (kind x union declared
//! type x constant reference), candidates that are settled and have nothing
//! to check per walk are joined to their parent (union-find), so a walk
//! over settled nodes costs about O(1). A file is indexed only once its
//! walks have taken 16 steps per flow node (sampled), so files with short
//! walks pay almost nothing.
//!
//! `GOPORT_FLOWSKIP`: unset or `1` on, `0` off, `verify` indexes every file,
//! tests each walk, then runs Go's walk anyway, and panics when the skip
//! would differ (result, counters of made things and effects, visited nodes,
//! depth). `GOPORT_FLOWSKIP_STATS=<file>` writes the process totals of the
//! tests to `<file>` (updated every 256 tests per checker).
//! `GOPORT_FLOWSKIP_BUILD_STEPS` and `GOPORT_FLOWSKIP_MIN_STEPS` set the
//! tuning values for A/B runs.

use crate::prelude::*;
use std::cell::Cell;
use std::hash::Hasher;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};

const NONE: u32 = u32::MAX;

/// Reference kinds (go-model-fix.md "Class").
const K_IDENT: u8 = 0;
const K_THIS: u8 = 1;
/// An access chain on an identifier root.
const K_ACC_IDENT: u8 = 2;
/// An access chain on `this`.
const K_ACC_THIS: u8 = 3;
const KINDS: usize = 4;

/// Mention and chain-site families. A `_U` family comes from a
/// discriminant role (`getDiscriminantPropertyAccess`, flow.go:1436), which
/// runs only when the declared type is a union.
const F_IDENT: u8 = 1;
const F_IDENT_U: u8 = 2;
const F_ACC: u8 = 3;
const F_ACC_U: u8 = 4;
/// Matched against each proper prefix of an access reference
/// (`containsMatchingReference`, flow.go:1841, and the `in` operand,
/// flow.go:523).
const F_ACCP: u8 = 5;
/// Chain sites: the root identifier of an access target or source whose
/// names equal the names of R (`C_M`, `C_M_U`) or of a proper access prefix
/// of R (`C_CM`) is resolved by Go (flow.go:1625-1633).
const C_M: u8 = 6;
const C_M_U: u8 = 7;
const C_CM: u8 = 8;

/// `c.inlineLevel < 5` (flow.go:386).
const MAX_INLINE: u8 = 5;

/// Root kinds of the next-pointer forest.
const ROOT_START: u8 = 1;
const ROOT_UNREACHABLE: u8 = 2;
const ROOT_BLOCKED: u8 = 3;

/// Candidate states per class.
const S_UNKNOWN: u8 = 0;
const S_JOINED: u8 = 1;
const S_EXTRAS: u8 = 2;
const S_BLOCKED: u8 = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlowSkipMode {
    Off,
    On,
    Verify,
}

/// `GOPORT_FLOWSKIP`, read once per process.
pub fn flow_skip_mode_from_env() -> FlowSkipMode {
    static MODE: OnceLock<FlowSkipMode> = OnceLock::new();
    *MODE.get_or_init(|| match std::env::var("GOPORT_FLOWSKIP").as_deref() {
        Ok("0") => FlowSkipMode::Off,
        Ok("verify") => FlowSkipMode::Verify,
        _ => FlowSkipMode::On,
    })
}


/// Why a test said no (stats only).
#[derive(Clone, Copy, Debug)]
#[repr(usize)]
enum Refusal {
    Kind,
    Inline,
    File,
    Counting,
    Short,
    Unreached,
    Mention,
    Chain,
    Depth,
    Unsettled,
    Blocked,
    Extras,
    Root,
    Cross,
}
const REFUSALS: usize = 14;
const REFUSAL_NAMES: [&str; REFUSALS] = [
    "kind",
    "inline",
    "file",
    "counting",
    "short",
    "unreached",
    "mention",
    "chain",
    "depth",
    "unsettled",
    "blocked",
    "extras",
    "root",
    "cross",
];

/// Counters of the tests of one checker (`GOPORT_FLOWSKIP_STATS`).
#[derive(Clone, Copy, Debug, Default)]
pub struct FlowSkipStats {
    pub tested: u64,
    pub skipped: u64,
    pub skipped_steps: u64,
    refused: [u64; REFUSALS],
    pub built: u64,
    pub built_nodes: u64,
    pub build_ns: u64,
}

const STAT_SLOTS: usize = 6 + REFUSALS;
static STATS_TOTAL: [AtomicU64; STAT_SLOTS] = [const { AtomicU64::new(0) }; STAT_SLOTS];

fn stats_path() -> Option<&'static str> {
    static PATH: OnceLock<Option<String>> = OnceLock::new();
    PATH.get_or_init(|| std::env::var("GOPORT_FLOWSKIP_STATS").ok())
        .as_deref()
}

impl FlowSkipStats {

}

/// Verify mode: what Go's walk did.
#[derive(Debug, Default)]
pub struct FlowSkipRecording {
    pub visited: Vec<FlowNodeId>,
    pub max_depth: i32,
}




enum FileState {
    /// Walks so far, and the estimated sum of their path lengths (every
    /// `SAMPLE_EVERY`th walk is measured, at most `DRY_CAP` steps).
    Counting {
        walks: u32,
        steps: u64,
    },
    Built(Box<FileIndex>),
    /// Not a static file: never indexed.
    Unsupported,
}

/// A measured walk counts at most this many steps.
const DRY_CAP: u32 = 256;
/// One walk in this many is measured.
const SAMPLE_EVERY: u32 = 4;

/// Steps of pi(u) to its root (no move to an outer function), at most
/// `DRY_CAP`: the length of Go's walk when nothing narrows.
fn dry_steps(flows: &[FlowNode], file: usize, u: usize) -> u32 {
    let mut x = u;
    let mut steps = 1;
    while steps < DRY_CAP {
        match next_of(flows, file, &flows[x]) {
            Ok((p, _)) => x = p as usize,
            Err(_) => break,
        }
        steps += 1;
    }
    steps
}

// ──────────────────────────────────────────────────────────────────────
// Keys
// ──────────────────────────────────────────────────────────────────────

/// A mention key: family, the property names from the top of the chain
/// down, then the root text (`this` for the keyword). Equal texts give
/// equal keys; a hash collision only makes a test refuse more.
fn chain_key(family: u8, names: &[&'static str], root: &str) -> u64 {
    let mut h = crate::rustc_hash::FxHasher::default();
    h.write_u8(family);
    for name in names {
        h.write(name.as_bytes());
        h.write_u8(0xff);
    }
    h.write_u8(0xfe);
    h.write(root.as_bytes());
    h.finish()
}

/// A chain-site key: family and the property names, without the root.
fn names_key(family: u8, names: &[&'static str]) -> u64 {
    let mut h = crate::rustc_hash::FxHasher::default();
    h.write_u8(family);
    h.write_u8(0xfd);
    for name in names {
        h.write(name.as_bytes());
        h.write_u8(0xff);
    }
    h.finish()
}


// ──────────────────────────────────────────────────────────────────────
// Roles: the parts of `narrowType` (flow.go:377-1060) that a reference can
// meet when nothing matches it, as events.
// ──────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug)]
enum Ev {
    /// `isMatchingReference(R, x)` with x a target (flow.go:1597). The flag
    /// marks a discriminant role (union declared type only).
    M(Node, bool),
    /// `optionalChainContainsReference(x, R)` (flow.go:1851).
    Oc(Node),
    /// `containsMatchingReference(R, x)` (flow.go:1841), and the match of
    /// `R.Expression()` with the `in` operand (flow.go:523). Access kinds.
    Cm(Node),
    /// An identifier in an N position at alias depth d < 5: M, resolved
    /// (flow.go:388), inline alias candidate (flow.go:389-396), else T.
    NIdent(Node, bool, u8),
    /// An identifier in a discriminant position (union only): resolved,
    /// discriminant alias candidate (flow.go:1477-1491).
    DIdent(Node),
    /// Union-making `&&`/`||` (flow.go:539-551) or `switch (true)`
    /// (flow.go:1069).
    Block,
}






/// Collects the events of one flow node or alias initializer.
struct Scan {
    strict: bool,
    out: Vec<Ev>,
}

impl Scan {







}

// ──────────────────────────────────────────────────────────────────────
// Meaning of the events for one reference kind
// ──────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug)]
enum Alias {
    /// N-position identifier: x, assumeTrue, alias depth.
    N(Node, bool, u8),
    /// D-position identifier.
    D(Node),
}

const ALL_BASE: u8 = 0b0101_0101;
const ALL_UNION: u8 = 0b1010_1010;

/// What the events of a node mean. Mention keys and chain sites carry
/// their family, which names the kinds that compare them, so they are kept
/// for every kind. Resolves and blocks carry the bits of the kinds (and
/// union part) that Go evaluates them for.
#[derive(Default)]
struct Out {
    mentions: Vec<u64>,
    chains: Vec<(u64, Node)>,
    resolves: Vec<(Node, u8)>,
    blocked: u8,
    aliases: Vec<Alias>,
}

impl Out {
    fn clear(&mut self) {
        self.mentions.clear();
        self.chains.clear();
        self.resolves.clear();
        self.blocked = 0;
        self.aliases.clear();
    }





}

// ──────────────────────────────────────────────────────────────────────
// The per-file index
// ──────────────────────────────────────────────────────────────────────

struct Cand {
    node: u32,
    /// The call of a call node, else nil.
    call: Node,
    /// `res_pool[res_start..res_end]`: identifiers Go resolves here, with
    /// the bits (`slot`) of the kinds that resolve them.
    res_start: u32,
    res_end: u32,
    /// Bits (`slot`): a key site or union-making operator.
    blocked: u8,
    alias_start: u32,
    alias_end: u32,
}

/// Extras of a settled candidate for one class: mentions and chain sites
/// that its aliases add, checked per walk.
#[derive(Default)]
struct Extras {
    mentions: Vec<u64>,
    chains: Vec<(u64, Node)>,
}

struct ClassState {
    next: Vec<u32>,
    state: Vec<u8>,
    extras: FxHashMap<u32, Extras>,
}

struct FileIndex {
    flows: &'static [FlowNode],
    file: usize,
    parent: Vec<u32>,
    root_kind: Vec<u8>,
    nest: Vec<u32>,
    steps: Vec<u32>,
    tin: Vec<u32>,
    stop: Vec<u32>,
    /// The root of each reached node's tree.
    root_of: Vec<u32>,
    cand_of: Vec<u32>,
    cands: Vec<Cand>,
    res_pool: Vec<(Node, u8)>,
    alias_pool: Vec<Alias>,
    /// Mentions: `(key, tin)` and `(key, tout)` of each mentioning node,
    /// sorted; `mention_range[key]` is the part of both lists with `key`.
    mention_in: Vec<u32>,
    mention_out: Vec<u32>,
    mention_range: FxHashMap<u64, (u32, u32)>,
    /// Chain sites sorted by key, then tin: key, tin, tout, root.
    chain_sites: Vec<(u64, u32, u32, Node)>,
    classes: Vec<Option<Box<ClassState>>>,
}

#[derive(Clone, Copy, Debug)]
struct Class {
    kind: u8,
    union: bool,
    /// `isConstantReference(R)`: 0 false, 1 true, 2 unknown (not pure yet).
    cr: u8,
}

impl Class {
    fn index(self) -> usize {
        usize::from(self.kind) * 6 + usize::from(self.union) * 3 + usize::from(self.cr)
    }
}

const CLASSES: usize = KINDS * 6;

/// Static next pointer of a flow node: the node Go's walk visits after it
/// when the node is inert (pi in go-model.md 2.2), and whether that step
/// nests. A root has kind `ROOT_*`.
fn next_of(flows: &[FlowNode], file: usize, flow: &FlowNode) -> Result<(u32, u32), u8> {
    let flags = flow.flags;
    let local = |id: FlowNodeId| -> Result<u32, u8> {
        if id.is_nil() || id.file_index() != file || id.local_index() >= flows.len() {
            Err(ROOT_BLOCKED)
        } else {
            Ok(id.local_index() as u32)
        }
    };
    if flags.intersects(FlowFlags::ASSIGNMENT | FlowFlags::CALL) {
        Ok((local(flow.antecedent)?, 0))
    } else if flags.intersects(FlowFlags::CONDITION | FlowFlags::SWITCH_CLAUSE) {
        Ok((local(flow.antecedent)?, 1))
    } else if flags.intersects(FlowFlags::BRANCH_LABEL) {
        match flow.antecedents.len() {
            0 => Err(ROOT_BLOCKED),
            1 => Ok((local(flow.antecedents[0])?, 0)),
            _ => {
                let first = local(flow.antecedents[0])?;
                let first_flow = &flows[first as usize];
                // The first empty switch clause is the bypass (flow.go:1260).
                let bypass = first_flow.flags.intersects(FlowFlags::SWITCH_CLAUSE)
                    && first_flow.antecedents[0].0 as u32 == first_flow.antecedents[1].0 as u32;
                if bypass {
                    Ok((local(flow.antecedents[1])?, 1))
                } else {
                    Ok((first, 1))
                }
            }
        }
    } else if flags.intersects(FlowFlags::LOOP_LABEL) {
        // A loop label with 2+ antecedents writes `flowLoopCache`
        // (flow.go:1325): not in v1.
        if flow.antecedents.len() == 1 {
            Ok((local(flow.antecedents[0])?, 0))
        } else {
            Err(ROOT_BLOCKED)
        }
    } else if flags.intersects(FlowFlags::ARRAY_MUTATION) {
        Ok((local(flow.antecedent)?, 0))
    } else if flags.intersects(FlowFlags::REDUCE_LABEL) {
        Err(ROOT_BLOCKED)
    } else if flags.intersects(FlowFlags::START) {
        Err(ROOT_START)
    } else {
        Err(ROOT_UNREACHABLE)
    }
}

fn is_candidate(flags: FlowFlags) -> bool {
    // Same test order as `get_type_at_flow_node`.
    flags.intersects(FlowFlags::ASSIGNMENT | FlowFlags::CALL)
        || flags.intersects(FlowFlags::CONDITION | FlowFlags::SWITCH_CLAUSE)
}

impl FileIndex {


    fn class_state(&mut self, class: Class) -> &mut ClassState {
        let slot = &mut self.classes[class.index()];
        slot.get_or_insert_with(|| {
            Box::new(ClassState {
                next: self.stop.clone(),
                state: vec![S_UNKNOWN; self.cands.len()],
                extras: FxHashMap::default(),
            })
        })
    }
}

/// Union-find with path halving.
fn find(next: &mut [u32], mut x: u32) -> u32 {
    while next[x as usize] != x {
        let p = next[x as usize];
        next[x as usize] = next[p as usize];
        x = p;
    }
    x
}


enum Settle {
    Joined,
    Extras(Extras),
    Unsettled,
    Blocked,
}

// ──────────────────────────────────────────────────────────────────────
// Checker side
// ──────────────────────────────────────────────────────────────────────

/// Values that a skipped walk must not change (verify mode).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Snapshot {
    types: u32,
    symbols: u32,
    signatures: u32,
    diagnostics: i32,
    suggestions: i32,
    union_of_unions: usize,
    flow_loop_cache: usize,
    last_flow_node: FlowNodeId,
    flow_node_reachable: usize,
    flow_analysis_disabled: bool,
    flow_invocation_count: i32,
    effects: u64,
    resolve_names: u64,
}


#[cfg(test)]
mod tests {
    use super::*;

    /// Checks `a.ts` (`source`) in a new project with `options` (JSON
    /// members of `compilerOptions`), with the skip in verify mode and every
    /// walk tested. Verify mode panics when a skipped walk differs from
    /// Go's walk. Returns the diagnostic codes and, per tested reference,
    /// its text, its position and whether it was skipped.
    fn check_with_skip(
        label: &str,
        source: &str,
        options: &str,
    ) -> (Vec<i32>, Vec<(String, i32, bool)>) {
        let dir =
            std::env::temp_dir().join(format!("ts_goport_flowskip_{label}_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.ts"), source).unwrap();
        std::fs::write(
            dir.join("tsconfig.json"),
            format!(
                r#"{{ "compilerOptions": {{ {options}, "target": "es2020", "types": [] }}, "files": ["a.ts"] }}"#
            ),
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
        let result = crate::program::with_type_checker_for_file(file, move |checker| {
            checker.flow_skip.mode = FlowSkipMode::Verify;
            checker.flow_skip.build_steps = 0;
            checker.flow_skip.min_steps = 0;
            checker.flow_skip.trace = Some(Vec::new());
            let ctx = crate::gostd::context::background();
            let codes: Vec<i32> = checker
                .get_diagnostics(&ctx, file, false)
                .iter()
                .map(|d| d.code())
                .collect();
            let trace = checker
                .flow_skip
                .trace
                .take()
                .unwrap()
                .into_iter()
                .map(|(node, skipped)| (get_text_of_node(node), node.end(), skipped))
                .collect();
            (codes, trace)
        });
        drop(scope);
        crate::program::release_program(program);
        result
    }

    /// Whether the test skipped the reference whose text is `text` and
    /// that ends at the end of the `n`th (from 0) match of `marker`.
    fn skipped_at(
        trace: &[(String, i32, bool)],
        source: &str,
        marker: &str,
        text: &str,
    ) -> Vec<bool> {
        let end = (source.find(marker).expect("marker") + marker.len()) as i32;
        trace
            .iter()
            .filter(|(t, e, _)| t == text && *e == end)
            .map(|(_, _, skipped)| *skipped)
            .collect()
    }

    /// Plain walks through conditions on other names are skipped, and
    /// verify mode finds Go's walk equal.
    #[test]
    fn plain_walks_are_skipped() {
        let source = "declare const c: boolean;\n\
            export function f(input: { a: number }, y: number) {\n\
            if (c) {} if (c) {} if (c) {} if (c) {} if (c) {}\n\
            const z = input.a + y;\n\
            return input;\n\
            }\n";
        let (codes, trace) = check_with_skip("plain", source, r#""strict": true"#);
        assert_eq!(codes, Vec::<i32>::new());
        assert_eq!(
            skipped_at(&trace, source, "return input", "input"),
            vec![true]
        );
        assert_eq!(skipped_at(&trace, source, "input.a", "input.a"), vec![true]);
    }

    /// Check counterexample 1a: an inline alias `p || q` with assumeTrue
    /// calls getUnionType (flow.go:549), which makes an origin union for
    /// `S | undefined`. The walk of `x` must not be skipped.
    #[test]
    fn union_making_alias_is_not_skipped() {
        let source = "type S = \"a\" | \"b\";\n\
            declare const p: boolean, q: boolean;\n\
            export function f(y: number, x: S | undefined) {\n\
            const ok = p || q;\n\
            if (ok) {}\n\
            y;\n\
            return x;\n\
            }\n";
        let (codes, trace) = check_with_skip("union_alias", source, r#""strict": true"#);
        assert_eq!(codes, Vec::<i32>::new());
        assert_eq!(skipped_at(&trace, source, "return x", "x"), vec![false]);
    }

    /// Check counterexample 1b: with an unresolved annotation, Go's walk
    /// returns errorType through getUnionType([E, E]); a skip would give
    /// `Missing` in the declaration.
    #[test]
    fn union_making_alias_on_error_type_is_not_skipped() {
        let source = "declare const p: boolean, q: boolean;\n\
            export function f(x: Missing) {\n\
            const ok = p || q;\n\
            if (!ok) throw 0;\n\
            x;\n\
            return x;\n\
            }\n";
        let (codes, trace) = check_with_skip("error_alias", source, r#""strict": true"#);
        assert_eq!(codes, vec![2304]);
        assert_eq!(skipped_at(&trace, source, "return x", "x"), vec![false]);
    }

    /// Check counterexample 2: `"b" in a` narrows `a.b` when its type has a
    /// missing type (flow.go:523, exactOptionalPropertyTypes).
    #[test]
    fn in_operand_prefix_is_not_skipped() {
        let source = "declare const a: { b?: string };\n\
            declare const z: boolean;\n\
            a.b;\n\
            if (z) {} if (z) {} if (z) {} if (z) {}\n\
            if (z) {} if (z) {} if (z) {} if (z) {}\n\
            if (\"b\" in a) {\n\
            const s: string = a.b;\n\
            }\n";
        let (codes, trace) = check_with_skip(
            "in_operand",
            source,
            r#""strict": true, "exactOptionalPropertyTypes": true"#,
        );
        assert_eq!(codes, Vec::<i32>::new());
        assert_eq!(
            skipped_at(&trace, source, "string = a.b", "a.b"),
            vec![false]
        );
    }

    /// Check gap 3: a key site in an inline alias initializer resolves the
    /// key on every walk (flow.go:1753).
    #[test]
    fn key_site_in_alias_is_not_skipped() {
        let source = "declare const o: { [k: string]: { p: number } | undefined };\n\
            declare const K: string;\n\
            export function f(x: number) {\n\
            const ok = o[K]?.p;\n\
            if (ok) {}\n\
            x;\n\
            return x;\n\
            }\n";
        let (codes, trace) = check_with_skip("key_alias", source, r#""strict": true"#);
        assert_eq!(codes, Vec::<i32>::new());
        assert_eq!(skipped_at(&trace, source, "return x", "x"), vec![false]);
    }

    /// `switch (true)` narrows by its case expressions (flow.go:1187).
    #[test]
    fn switch_true_is_not_skipped() {
        let source = "export function f(x: string | number, y: number) {\n\
            switch (true) { case typeof x === \"string\": break; }\n\
            y;\n\
            return x;\n\
            }\n";
        let (codes, trace) = check_with_skip("switch_true", source, r#""strict": true"#);
        assert_eq!(codes, Vec::<i32>::new());
        assert_eq!(skipped_at(&trace, source, "return x", "x"), vec![false]);
    }
}
