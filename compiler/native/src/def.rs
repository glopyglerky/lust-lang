// Stable definition identity (docs/ARCH.md section 1.7, serial refactor S2).
//
// Two forms, and they must not mix:
// - Session-local: `CrateId` and `LocalDefId` are dense array indexes inside one
//   compiler session. They change when a session restarts or items move. They must
//   never enter a disk key, a slot key, or a fuzzer transcript.
// - Stable: `(StableCrateHash, DefPathHash)` names one item across sessions and
//   edits. Only this pair (`DiskDefId`) may enter disk keys and slot keys
//   (docs/ARCH.md sections 1.7 and 8.4).
//
// `DefId` is the in-memory form: a session-local crate plus the stable path hash.
// A `DefId` as a whole must never enter a disk key, because its `krate` field is
// session-local. Translate it with `Program::disk_id` at the write boundary.
//
// A rename changes an item's hash. That is intended: a rename is a delete plus an
// insert for caches. Hashes carry no generic arguments, no types, and no span.
use std::hash::{Hash, Hasher};

/// Hasher for stable def identity only. The FxHasher used for hash maps
/// and scan hashes keeps too little entropy between similar short
/// inputs: its rotate-xor-multiply step makes sequential names collide
/// at high rates (`filler_19` and `filler_92` hash equal for many crate
/// seeds, measured above 10 percent over random states), which turns
/// valid crates into internal DefPathHash collision errors. This mixer
/// folds each word with a 128-bit multiply (wyhash's mum step), so
/// every input bit reaches every output bit. Values differ from the
/// old FxHash-based ones for the same inputs: persisted slot manifests
/// and disk cache keys from older builds simply miss and rebuild.
struct DefHasher(u64);

#[inline]
fn mum(a: u64, b: u64) -> u64 {
    let r = (a as u128).wrapping_mul(b as u128);
    (r as u64) ^ ((r >> 64) as u64)
}

impl DefHasher {
    fn new() -> DefHasher {
        DefHasher(0x243f_6a88_85a3_08d3)
    }
    fn add(&mut self, w: u64) {
        self.0 = mum(self.0 ^ 0x9e37_79b9_7f4a_7c15, w ^ self.0);
    }
}

impl Hasher for DefHasher {
    fn write(&mut self, bytes: &[u8]) {
        let (words, rest) = bytes.as_chunks::<8>();
        for w in words {
            self.add(u64::from_le_bytes(*w));
        }
        for &b in rest {
            self.add(b as u64);
        }
    }
    fn write_u8(&mut self, i: u8) {
        self.add(i as u64);
    }
    fn write_u32(&mut self, i: u32) {
        self.add(i as u64);
    }
    fn write_u64(&mut self, i: u64) {
        self.add(i);
    }
    fn write_usize(&mut self, i: usize) {
        self.add(i as u64);
    }
    fn finish(&self) -> u64 {
        mum(self.0 ^ 0xa076_1d64_78bd_642f, self.0)
    }
}

/// One crate in this session. Dense, session-local. Today a session holds one
/// crate, so every def has `LOCAL_CRATE`; multi-crate sessions (S6) assign more.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CrateId(pub u32);

/// The session's one crate today.
pub const LOCAL_CRATE: CrateId = CrateId(0);

/// The crate disambiguator hash: stable across sessions. It covers the package
/// identity and the target (kind, name, and the `:test` distinction), per
/// docs/ARCH.md section 1.7 steps 1 and 2.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StableCrateHash(pub u64);

/// The stable hash of one item's definition path. Stable across sessions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DefPathHash(pub u64);

/// One definition in memory: which crate (session-local), and which item in it
/// (stable). Never written to disk as-is; see `DiskDefId`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DefId {
    pub krate: CrateId,
    pub hash: DefPathHash,
}

impl DefId {
    /// A dummy id for records whose real def does not exist yet. Today only
    /// `diag::Expansion` uses it: no macro defs exist until W-MACRO lands.
    #[allow(dead_code)] // First non-test record is a W-MACRO expansion.
    pub const PLACEHOLDER: DefId = DefId { krate: LOCAL_CRATE, hash: DefPathHash(0) };
}

/// A definition's dense index inside one crate, for one session. This is the fast
/// array index the checker and codegen use (it replaces the old `FnId`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LocalDefId(pub u32);

/// The disk form of a `DefId`: the only def identity that may enter disk keys,
/// slot keys, or fuzzer transcripts (docs/ARCH.md sections 1.7 and 8.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DiskDefId {
    pub krate: StableCrateHash,
    pub hash: DefPathHash,
}

impl DiskDefId {
    /// The 16-byte little-endian form used by the slot manifest encoding.
    pub fn to_bytes(self) -> [u8; 16] {
        let mut out = [0u8; 16];
        out[..8].copy_from_slice(&self.krate.0.to_le_bytes());
        out[8..].copy_from_slice(&self.hash.0.to_le_bytes());
        out
    }

    /// Reads a value written by `to_bytes`. None on a short input.
    pub fn from_bytes(b: &[u8]) -> Option<DiskDefId> {
        let krate = u64::from_le_bytes(b.get(..8)?.try_into().ok()?);
        let hash = u64::from_le_bytes(b.get(8..16)?.try_into().ok()?);
        Some(DiskDefId { krate: StableCrateHash(krate), hash: DefPathHash(hash) })
    }
}

/// Both identities of the crate a compile works on: the session-local id and the
/// stable hash. The driver makes one per crate session from its session key
/// (S6); W-RESOLVE supplies the real package identity from the Cargo graph.
#[derive(Clone, Copy, Debug)]
pub struct CrateIds {
    pub local: CrateId,
    pub stable: StableCrateHash,
}

/// The kind tag of an item in its def path. Tags are hashed, so they are part of
/// stable identity: add new kinds at the end only, never renumber.
#[allow(dead_code)] // Adt and Macro tags are reserved now; their defs land in S3a and W-MACRO.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DefKind {
    Fn = 1,
    Method = 2,
    Impl = 3,
    Adt = 4,
    Macro = 5,
    // Added by W-RESOLVE for the item kinds resolution binds (section 1.3).
    Const = 6,
    Static = 7,
    TypeAlias = 8,
    Module = 9,
    // Added at the end for W-TRAITS-CORE (wave A): trait and
    // associated-item defs. Existing hashes do not move.
    Trait = 10,
    Assoc = 11,
    // Added at the end for closures (ARCH 1.7: parent fn hash plus
    // the closure's ordinal in that body, in token order).
    Closure = 12,
}

impl DefKind {
    /// The short label used in canonical path text (collision checks, debug).
    pub fn label(self) -> &'static str {
        match self {
            DefKind::Fn => "fn",
            DefKind::Method => "method",
            DefKind::Impl => "impl",
            DefKind::Adt => "adt",
            DefKind::Macro => "macro",
            DefKind::Const => "const",
            DefKind::Static => "static",
            DefKind::TypeAlias => "type",
            DefKind::Module => "mod",
            DefKind::Trait => "trait",
            DefKind::Assoc => "assoc",
            DefKind::Closure => "closure",
        }
    }
}

/// The stable crate hash for a package and target (section 1.7 steps 1 and 2).
/// `package_id` is the Cargo package id string (name, version, source) once
/// W-RESOLVE supplies it; for a path package it carries the workspace-relative
/// manifest path in place of the source. `test` marks the same target compiled
/// as a test crate: test and non-test defs must never share identity.
pub fn stable_crate_hash(package_id: &str, target_kind: &str, target_name: &str, test: bool) -> StableCrateHash {
    let mut h = DefHasher::new();
    "lust-crate-v1".hash(&mut h);
    package_id.hash(&mut h);
    target_kind.hash(&mut h);
    target_name.hash(&mut h);
    (if test { ":test" } else { "" }).hash(&mut h);
    StableCrateHash(h.finish())
}

/// The stable crate hash of today's synthetic one-file bin crate. There is no
/// Cargo graph yet, so the file path (as the request names it, which is also the
/// daemon's session key) stands in for the package id, and the file stem is the
/// bin target name. `test` is set for `lust test` harness builds. W-RESOLVE
/// replaces this with the real package id and target when Cargo loading lands.
/// Since S6 the driver hashes the request's session key instead; this helper
/// stays as the reference form the synthetic key must match (its tests and
/// `driver::session`'s test pin the two together).
#[allow(dead_code)]
pub fn synthetic_crate_hash(file_path: &str, test: bool) -> StableCrateHash {
    let stem = file_path.rsplit('/').next().unwrap_or(file_path);
    let stem = stem.strip_suffix(".rs").unwrap_or(stem);
    stable_crate_hash(&format!("file:{file_path}"), "bin", stem, test)
}

/// Hashes one item path: crate, module segments (name strings, never `Sym`),
/// kind tag, item name, and a disambiguator for same-name items (section 1.7
/// steps 3 to 5). Returns the hash and the canonical path text. The text is
/// what collision checks compare: two distinct texts with one hash are a
/// collision; the same text twice is a duplicate definition (the checker's error).
pub fn item_path(krate: StableCrateHash, module: &[&str], kind: DefKind, name: &str, disamb: u32) -> (DefPathHash, String) {
    let mut text = String::new();
    for seg in module {
        text.push_str(seg);
        text.push_str("::");
    }
    text.push_str(&format!("{}:{name}#{disamb}", kind.label()));
    (item_hash(krate, module, kind, name, disamb), text)
}

/// The hash half of `item_path`, without building the text. Callers that run
/// per compile and keep no path table (`collect`) use this form.
pub fn item_hash(krate: StableCrateHash, module: &[&str], kind: DefKind, name: &str, disamb: u32) -> DefPathHash {
    let mut h = DefHasher::new();
    krate.0.hash(&mut h);
    (module.len() as u64).hash(&mut h);
    for seg in module {
        seg.hash(&mut h);
    }
    (kind as u64).hash(&mut h);
    name.hash(&mut h);
    (disamb as u64).hash(&mut h);
    DefPathHash(h.finish())
}

/// Hashes an item under a parent def (a method under its impl, and later a
/// closure under its fn). The parent hash already covers the crate and module,
/// so only the kind, name, and disambiguator are added.
pub fn child_path(parent: DefPathHash, parent_text: &str, kind: DefKind, name: &str, disamb: u32) -> (DefPathHash, String) {
    (child_hash(parent, kind, name, disamb), format!("{parent_text}::{}:{name}#{disamb}", kind.label()))
}

/// The hash half of `child_path`. See `item_hash`.
pub fn child_hash(parent: DefPathHash, kind: DefKind, name: &str, disamb: u32) -> DefPathHash {
    let mut h = DefHasher::new();
    parent.0.hash(&mut h);
    (kind as u64).hash(&mut h);
    name.hash(&mut h);
    (disamb as u64).hash(&mut h);
    DefPathHash(h.finish())
}

/// A free function's path.
pub fn free_fn_path(krate: StableCrateHash, module: &[&str], name: &str) -> (DefPathHash, String) {
    item_path(krate, module, DefKind::Fn, name, 0)
}

/// An impl block's path. `header` is the impl's canonical header text and `ordinal`
/// counts impls with the same header earlier in the crate, in item order (`impl_ids`).
/// Body edits keep the id; a header edit makes a new one (section 1.7).
pub fn impl_path(krate: StableCrateHash, module: &[&str], header: &str, ordinal: u32) -> (DefPathHash, String) {
    item_path(krate, module, DefKind::Impl, header, ordinal)
}

/// A method's path: parent impl hash plus the method name (section 1.7).
pub fn method_path(impl_hash: DefPathHash, impl_text: &str, name: &str) -> (DefPathHash, String) {
    child_path(impl_hash, impl_text, DefKind::Method, name, 0)
}

/// A free function's hash. See `item_hash`.
pub fn free_fn_hash(krate: StableCrateHash, module: &[&str], name: &str) -> DefPathHash {
    item_hash(krate, module, DefKind::Fn, name, 0)
}

/// An impl block's hash. See `impl_path` for the header and ordinal rules.
pub fn impl_hash(krate: StableCrateHash, module: &[&str], header: &str, ordinal: u32) -> DefPathHash {
    item_hash(krate, module, DefKind::Impl, header, ordinal)
}

/// A method's hash. See `item_hash`.
pub fn method_hash(impl_hash: DefPathHash, name: &str) -> DefPathHash {
    child_hash(impl_hash, DefKind::Method, name, 0)
}

/// One impl block's def path and its methods' (`impl_ids`).
pub struct ImplIds {
    /// The impl's own path: its header with its ordinal among impls of the same header.
    pub path: (DefPathHash, String),
    /// Each method's path, in `ImplDef::fns` order.
    pub methods: Vec<(DefPathHash, String)>,
}

/// The identity of every impl block, by item index (None for other items). The header is
/// the impl's self type and, for a trait impl, its trait, as canonical text with generic
/// type args (`Clone for Foo<T>`, `Foo<u8>` for an inherent impl; `header_ty_text`). The
/// impl's own path takes an ordinal among impls of the same header, in item order. A
/// method's path does not: it is the header's (ordinal 0) plus the method name, with an
/// ordinal only among methods of the same header and name (headers that differ only in a
/// form the text leaves out). So inserting an impl, splitting an inherent impl into
/// blocks, or moving one to another file renames no method of another block, and none of
/// their instances: a positional number here once renamed every method of a type when an
/// impl of it was added, and every caller and inline copy recompiled (rel-cache, merge 1
/// of the tsgo replay: 285 units). The driver's `fn_keys` and `collect` both use this.
///
/// The header texts and the paths are made on the pool (tsgo: a few ms a call, several
/// calls a build); only the ordinals count in item order, here.
pub fn impl_ids(krate: StableCrateHash, ast: &crate::ast::Ast, interner: &crate::intern::Interner) -> Vec<Option<ImplIds>> {
    let headers: Vec<Option<String>> = crate::pool::par_map(&ast.items, |item| {
        let crate::ast::ItemKind::Impl(imp) = &item.kind else { return None };
        let mut header = String::new();
        if let Some(t) = &imp.trait_ref {
            if imp.is_negative {
                header.push('!');
            }
            path_text(ast, interner, &t.path, &mut header);
            header.push_str(" for ");
        }
        header_ty_text(ast, interner, imp.self_ty, &mut header);
        Some(header)
    });
    let mut impl_ordinals: crate::intern::FxMap<&str, u32> = Default::default();
    let mut method_ordinals: crate::intern::FxMap<(&str, &str), u32> = Default::default();
    let ordinals: Vec<Option<(u32, Vec<u32>)>> = ast
        .items
        .iter()
        .zip(&headers)
        .map(|(item, header)| {
            let (crate::ast::ItemKind::Impl(imp), Some(header)) = (&item.kind, header) else { return None };
            let next = |n: &mut u32| {
                *n += 1;
                *n - 1
            };
            let ordinal = next(impl_ordinals.entry(header.as_str()).or_insert(0));
            let methods = imp.fns.iter().map(|f| next(method_ordinals.entry((header.as_str(), interner.get(f.name))).or_insert(0))).collect();
            Some((ordinal, methods))
        })
        .collect();
    let at: Vec<usize> = (0..ast.items.len()).collect();
    crate::pool::par_map(&at, |&i| {
        let (crate::ast::ItemKind::Impl(imp), Some(header), Some((ordinal, method_ordinals))) = (&ast.items[i].kind, &headers[i], &ordinals[i]) else {
            return None;
        };
        let path = impl_path(krate, &[], header, *ordinal);
        let (parent, parent_text) = impl_path(krate, &[], header, 0);
        let methods = imp.fns.iter().zip(method_ordinals).map(|(f, &n)| child_path(parent, &parent_text, DefKind::Method, interner.get(f.name), n)).collect();
        Some(ImplIds { path, methods })
    })
}

/// The canonical text of a type in an impl header (`impl_ids`): paths with their generic
/// type args, references, pointers, tuples, slices, arrays; lifetimes left out, as regions
/// do not tell impls apart. A form it does not spell out (a const arg, an array length, a
/// fn pointer, `dyn`, `impl`) prints as `_`; impls that differ only there are told apart
/// by ordinals.
fn header_ty_text(ast: &crate::ast::Ast, interner: &crate::intern::Interner, t: crate::ast::TyId, out: &mut String) {
    use crate::ast::TyExpr;
    match ast.ty(t) {
        TyExpr::Path(segs) => path_text(ast, interner, segs, out),
        TyExpr::Ref { mutable, inner, .. } => {
            out.push_str(if *mutable { "&mut " } else { "&" });
            header_ty_text(ast, interner, *inner, out);
        }
        TyExpr::Ptr { mutable, inner } => {
            out.push_str(if *mutable { "*mut " } else { "*const " });
            header_ty_text(ast, interner, *inner, out);
        }
        TyExpr::Unit => out.push_str("()"),
        TyExpr::Tuple(ts) => {
            out.push('(');
            for (i, t) in ts.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                header_ty_text(ast, interner, *t, out);
            }
            out.push(')');
        }
        TyExpr::Slice(t) => {
            out.push('[');
            header_ty_text(ast, interner, *t, out);
            out.push(']');
        }
        TyExpr::Array { inner, .. } => {
            out.push('[');
            header_ty_text(ast, interner, *inner, out);
            out.push_str("; _]");
        }
        TyExpr::Paren(t) => header_ty_text(ast, interner, *t, out),
        TyExpr::Never => out.push('!'),
        _ => out.push('_'),
    }
}

/// A path's canonical text for `header_ty_text`: segments joined by `::`, each with its
/// generic args.
fn path_text(ast: &crate::ast::Ast, interner: &crate::intern::Interner, segs: &[crate::ast::PathSeg], out: &mut String) {
    use crate::ast::{GenericArg, GenericArgs};
    for (i, s) in segs.iter().enumerate() {
        if i > 0 {
            out.push_str("::");
        }
        out.push_str(interner.get(s.name));
        match &s.args {
            GenericArgs::None => {}
            GenericArgs::Angle(args) => {
                out.push('<');
                for (k, a) in args.iter().filter(|a| !matches!(a, GenericArg::Lifetime(_))).enumerate() {
                    if k > 0 {
                        out.push_str(", ");
                    }
                    match a {
                        GenericArg::Type(t) => header_ty_text(ast, interner, *t, out),
                        GenericArg::Binding { name, .. } => {
                            out.push_str(interner.get(*name));
                            out.push_str(" = _");
                        }
                        _ => out.push('_'),
                    }
                }
                out.push('>');
            }
            GenericArgs::Parenthesized { inputs, output } => {
                out.push('(');
                for (k, t) in inputs.iter().enumerate() {
                    if k > 0 {
                        out.push_str(", ");
                    }
                    header_ty_text(ast, interner, *t, out);
                }
                out.push(')');
                if let Some(o) = output {
                    out.push_str(" -> ");
                    header_ty_text(ast, interner, *o, out);
                }
            }
        }
    }
}

/// The path of each trait default method, by its body block: the trait's
/// path (its name, with its ordinal among same-name traits as the
/// disambiguator, as `collect_trait_defs` numbers them) plus the method name.
/// These parent the items nested in default bodies, next to the top-level fns
/// and impl methods (`collect::top_fn_hashes`, the driver's `fn_keys`).
pub fn trait_body_paths(
    krate: StableCrateHash,
    ast: &crate::ast::Ast,
    interner: &crate::intern::Interner,
) -> Vec<(crate::ast::BlockId, DefPathHash, String)> {
    let mut ordinals: crate::intern::FxMap<crate::intern::Sym, u32> = Default::default();
    let mut out = Vec::new();
    for item in &ast.items {
        let crate::ast::ItemKind::Trait(t) = &item.kind else { continue };
        let ordinal = ordinals.entry(t.name).or_insert(0);
        let (thash, ttext) = item_path(krate, &[], DefKind::Trait, interner.get(t.name), *ordinal);
        *ordinal += 1;
        for f in &t.fns {
            if let Some(b) = f.body {
                let (hash, text) = child_path(thash, &ttext, DefKind::Method, interner.get(f.name), 0);
                out.push((b, hash, text));
            }
        }
    }
    out
}

/// The fn whose body holds a fn nested in a body (see `nested_fn_parents`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NestParent {
    /// A top-level fn, an impl method, or a trait default method, by its
    /// body block.
    Top(crate::ast::BlockId),
    /// Another nested fn, by its `Ast::stmt_items` index.
    Nested(u32),
    /// No fn body holds it by span: a fn from a macro expansion, or one in a
    /// const or static initializer.
    Unknown,
}

/// The parent and the disambiguator of each fn nested in a body (section 1.7),
/// by `Ast::stmt_items` index; other entries are `(Unknown, 0)`. The parent is
/// the innermost top-level fn, impl method, trait default method, or nested fn
/// whose body span holds the item's span. The disambiguator counts the same-name fns of that parent
/// before it, in body item order. A nested fn's identity is its parent's
/// identity plus its name and disambiguator, so an item added to one body does
/// not move the identity of the fns nested in other bodies. Spans only find the
/// parent; they never enter a hash. Every answer is unique (a parent identity
/// is unique, and the disambiguator splits same-name fns of one parent), so a
/// parent missed (macro spans) costs only reuse. `collect` (def ids) and the
/// driver (slot keys) both call this, so the two agree.
pub fn nested_fn_parents(ast: &crate::ast::Ast) -> Vec<(NestParent, u32)> {
    use crate::ast::ItemKind;
    let mut out = vec![(NestParent::Unknown, 0u32); ast.stmt_items.len()];
    if !ast.stmt_items.iter().any(|it| matches!(it.kind, ItemKind::Fn(_))) {
        return out;
    }
    // A fn body span opens a scope (`Some(parent)`); a nested fn item span
    // (`None`, with its stmt index) takes the innermost open scope. Sorted
    // by file, start, and longest first, the open scopes form a stack.
    let mut spans: Vec<(crate::diag::Span, Option<NestParent>, u32)> = Vec::new();
    let body = |spans: &mut Vec<_>, f: &crate::ast::FnDef, nested: Option<u32>| {
        if let Some(b) = f.body {
            let parent = nested.map_or(NestParent::Top(b), NestParent::Nested);
            spans.push((ast.block(b).span, Some(parent), 0));
        }
    };
    for item in &ast.items {
        match &item.kind {
            ItemKind::Fn(f) => body(&mut spans, f, None),
            ItemKind::Impl(imp) => imp.fns.iter().for_each(|f| body(&mut spans, f, None)),
            ItemKind::Trait(t) => t.fns.iter().for_each(|f| body(&mut spans, f, None)),
            _ => {}
        }
    }
    for (j, it) in ast.stmt_items.iter().enumerate() {
        if let ItemKind::Fn(f) = &it.kind {
            body(&mut spans, f, Some(j as u32));
            spans.push((it.span, None, j as u32));
        }
    }
    spans.retain(|s| s.0.ctxt == crate::diag::CtxtId::ROOT);
    spans.sort_by_key(|s| (s.0.file.0, s.0.lo, std::cmp::Reverse(s.0.hi)));
    let mut open: Vec<(crate::diag::Span, NestParent)> = Vec::new();
    for (span, parent, j) in spans {
        while open.last().is_some_and(|(s, _)| s.file != span.file || s.hi <= span.lo) {
            open.pop();
        }
        match parent {
            Some(p) => open.push((span, p)),
            None => {
                if let Some(&(s, p)) = open.last()
                    && span.hi <= s.hi
                {
                    out[j as usize].0 = p;
                }
            }
        }
    }
    let mut seen: crate::intern::FxMap<(NestParent, crate::intern::Sym), u32> = Default::default();
    for (j, it) in ast.stmt_items.iter().enumerate() {
        if let ItemKind::Fn(f) = &it.kind {
            let n = seen.entry((out[j].0, f.name)).or_insert(0);
            out[j].1 = *n;
            *n += 1;
        }
    }
    out
}

/// The identity of each fn nested in a body, by `Ast::stmt_items` index (None
/// for other items), folded down from `nested_fn_parents`: `child(parent id,
/// name, disambiguator)`, or `fallback(segment, name)` when the parent is
/// unknown, where the segment is the body item index (unique, but it moves
/// when an earlier body gains or loses an item). `top` gives the identity of
/// a top-level fn or impl method by its body block. `collect` folds hashes;
/// the driver folds hashes with path texts. Parents end after their nested
/// fns, so they come later in body item order and fold first.
pub fn nested_fn_ids<T: Clone>(
    ast: &crate::ast::Ast,
    interner: &crate::intern::Interner,
    top: impl Fn(crate::ast::BlockId) -> Option<T>,
    child: impl Fn(&T, &str, u32) -> T,
    fallback: impl Fn(&str, &str) -> T,
) -> Vec<Option<T>> {
    let nest = nested_fn_parents(ast);
    let mut ids: Vec<Option<T>> = vec![None; ast.stmt_items.len()];
    for j in (0..ast.stmt_items.len()).rev() {
        let crate::ast::ItemKind::Fn(f) = &ast.stmt_items[j].kind else { continue };
        let name = interner.get(f.name);
        let parent = match nest[j].0 {
            NestParent::Top(b) => top(b),
            NestParent::Nested(k) if k as usize > j => ids[k as usize].clone(),
            _ => None,
        };
        ids[j] = Some(match parent {
            Some(p) => child(&p, name, nest[j].1),
            None => fallback(&format!("#nested{j}"), name),
        });
    }
    ids
}

/// The name a type item inside a function body binds (a struct,
/// enum, trait, or type alias), if this item is one.
fn type_item_name(kind: &crate::ast::ItemKind) -> Option<crate::intern::Sym> {
    use crate::ast::ItemKind;
    match kind {
        ItemKind::Struct(s) => Some(s.name),
        ItemKind::Enum(e) => Some(e.name),
        ItemKind::Trait(t) => Some(t.name),
        ItemKind::Type(a) => Some(a.name),
        _ => None,
    }
}

/// True when some function body holds a type item. Callers skip the
/// fn hash work for body type identities when it is false (most
/// programs; the perf gate's plain inputs).
pub fn has_body_type_items(ast: &crate::ast::Ast) -> bool {
    ast.stmt_items.iter().any(|it| type_item_name(&it.kind).is_some())
}

/// The `DefKind` of one type item inside a function body.
pub fn type_item_kind(kind: &crate::ast::ItemKind) -> Option<DefKind> {
    use crate::ast::ItemKind;
    match kind {
        ItemKind::Struct(_) | ItemKind::Enum(_) | ItemKind::Union(_) => Some(DefKind::Adt),
        ItemKind::Trait(_) => Some(DefKind::Trait),
        ItemKind::Type(_) => Some(DefKind::TypeAlias),
        _ => None,
    }
}

/// The parent and disambiguator of each type item inside a function
/// body, by `Ast::stmt_items` index, like `nested_fn_parents` (the
/// span walk is the same; the disambiguator counts the same-name
/// type items of that parent).
pub fn nested_type_parents(ast: &crate::ast::Ast) -> Vec<(NestParent, u32)> {
    use crate::ast::ItemKind;
    let mut out = vec![(NestParent::Unknown, 0u32); ast.stmt_items.len()];
    if !has_body_type_items(ast) {
        return out;
    }
    let mut spans: Vec<(crate::diag::Span, Option<NestParent>, u32)> = Vec::new();
    let body = |spans: &mut Vec<_>, f: &crate::ast::FnDef, nested: Option<u32>| {
        if let Some(b) = f.body {
            let parent = nested.map_or(NestParent::Top(b), NestParent::Nested);
            spans.push((ast.block(b).span, Some(parent), 0));
        }
    };
    for item in &ast.items {
        match &item.kind {
            ItemKind::Fn(f) => body(&mut spans, f, None),
            ItemKind::Impl(imp) => imp.fns.iter().for_each(|f| body(&mut spans, f, None)),
            ItemKind::Trait(t) => t.fns.iter().for_each(|f| body(&mut spans, f, None)),
            _ => {}
        }
    }
    for (j, it) in ast.stmt_items.iter().enumerate() {
        if let ItemKind::Fn(f) = &it.kind {
            body(&mut spans, f, Some(j as u32));
        }
        if type_item_name(&it.kind).is_some() {
            spans.push((it.span, None, j as u32));
        }
    }
    spans.retain(|s| s.0.ctxt == crate::diag::CtxtId::ROOT);
    spans.sort_by_key(|s| (s.0.file.0, s.0.lo, std::cmp::Reverse(s.0.hi)));
    let mut open: Vec<(crate::diag::Span, NestParent)> = Vec::new();
    for (span, parent, j) in spans {
        while open.last().is_some_and(|(s, _)| s.file != span.file || s.hi <= span.lo) {
            open.pop();
        }
        match parent {
            Some(p) => open.push((span, p)),
            None => {
                if let Some(&(s, p)) = open.last()
                    && span.hi <= s.hi
                {
                    out[j as usize].0 = p;
                }
            }
        }
    }
    let mut seen: crate::intern::FxMap<(NestParent, crate::intern::Sym), u32> = Default::default();
    for (j, it) in ast.stmt_items.iter().enumerate() {
        if let Some(name) = type_item_name(&it.kind) {
            let n = seen.entry((out[j].0, name)).or_insert(0);
            out[j].1 = *n;
            *n += 1;
        }
    }
    out
}

/// The identity of each type item inside a function body, folded
/// like `nested_fn_ids`: `child(parent id, name, disambiguator)`
/// with the item's kind folded by the caller's closure through
/// `kind`, or `fallback(segment, name)`. A type item's parent fn
/// identity comes from `nested_fn_ids` over the same parents, so
/// `top` and the nested-fn fold must agree with that call.
pub fn nested_type_ids<T: Clone>(
    ast: &crate::ast::Ast,
    interner: &crate::intern::Interner,
    parent_of: impl Fn(NestParent) -> Option<T>,
    child: impl Fn(&T, DefKind, &str, u32) -> T,
    fallback: impl Fn(&str, DefKind, &str) -> T,
) -> Vec<Option<T>> {
    let nest = nested_type_parents(ast);
    let mut ids: Vec<Option<T>> = vec![None; ast.stmt_items.len()];
    for (j, it) in ast.stmt_items.iter().enumerate() {
        let (Some(name), Some(kind)) = (type_item_name(&it.kind), type_item_kind(&it.kind)) else { continue };
        let name = interner.get(name);
        ids[j] = Some(match parent_of(nest[j].0) {
            Some(p) => child(&p, kind, name, nest[j].1),
            None => fallback(&format!("#nested{j}"), kind, name),
        });
    }
    ids
}

/// Two distinct def paths with one hash. This is an internal error, not a user
/// error: the caller stops the compile (the driver answers with code 3). No
/// collision-resolution scheme is added; the check makes the 64-bit hash safe
/// (section 1.7).
#[derive(Debug)]
pub struct Collision {
    pub hash: DefPathHash,
    pub first: String,
    pub second: String,
}

impl std::fmt::Display for Collision {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "internal: DefPathHash collision ({:#018x}) between `{}` and `{}`", self.hash.0, self.first, self.second)
    }
}

/// Records `path` under its disk identity in a path table. The same path again
/// is fine (a duplicate definition, reported later as L0106). A different path
/// with the same identity is a `Collision`. The same hash under a different
/// crate is a different identity, not a collision.
pub fn insert_path(paths: &mut crate::intern::FxMap<DiskDefId, Box<str>>, key: DiskDefId, path: &str) -> Result<(), Collision> {
    match paths.get(&key) {
        Some(old) if &**old != path => Err(Collision { hash: key.hash, first: old.to_string(), second: path.to_string() }),
        Some(_) => Ok(()),
        None => {
            paths.insert(key, path.into());
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intern::FxMap;

    fn krate() -> StableCrateHash {
        synthetic_crate_hash("src/main.rs", false)
    }

    /// The same inputs give the same hash in every call (that is, every session):
    /// only name strings enter the hash, never session-local ids.
    #[test]
    fn hashes_are_stable_and_input_sensitive() {
        let k = krate();
        let (a, at) = free_fn_path(k, &[], "run");
        let (b, bt) = free_fn_path(krate(), &[], "run");
        assert_eq!((a, at.as_str()), (b, bt.as_str()));
        // Each input named in section 1.7 changes the hash on its own.
        assert_ne!(a, free_fn_path(k, &[], "walk").0, "name");
        assert_ne!(a, free_fn_path(k, &["sub"], "run").0, "module");
        assert_ne!(a, item_path(k, &[], DefKind::Adt, "run", 0).0, "kind");
        assert_ne!(a, item_path(k, &[], DefKind::Fn, "run", 1).0, "disambiguator");
        assert_ne!(a, free_fn_path(synthetic_crate_hash("src/main.rs", true), &[], "run").0, "test target");
        assert_ne!(a, free_fn_path(synthetic_crate_hash("src/other.rs", false), &[], "run").0, "crate");
        assert_ne!(a, free_fn_path(stable_crate_hash("file:src/main.rs", "lib", "main", false), &[], "run").0, "target kind");
    }

    /// A method's identity is its parent impl plus its name: same-named methods
    /// on different types, and in different same-header impl blocks, differ.
    /// A method's id follows its impl's header (self type and trait, with generic args) and
    /// its own name, never the impl's place: impls inserted before it, an inherent impl
    /// split into blocks, or a trait impl moved elsewhere in the crate keep every method's
    /// id, and same-name methods of impls for other args stay apart.
    #[test]
    fn method_ids_do_not_depend_on_impl_order() {
        let ids = |src: &str| -> Vec<(String, DefPathHash)> {
            let interner = crate::intern::Interner::default();
            let known = crate::intern::Known::new(&interner);
            let lexed = crate::lexer::lex(src, crate::diag::FileId(0), 0, &interner).unwrap();
            let ast = crate::parser::parse(&lexed, &interner, &known).unwrap();
            let mut out = Vec::new();
            for (item, ids) in ast.items.iter().zip(impl_ids(StableCrateHash(7), &ast, &interner)) {
                if let (crate::ast::ItemKind::Impl(imp), Some(ids)) = (&item.kind, ids) {
                    out.extend(imp.fns.iter().zip(ids.methods).map(|(f, (h, _))| (interner.get(f.name).to_string(), h)));
                }
            }
            out
        };
        let base = ids("struct Foo<T>(T);\nimpl Foo<u8> { fn new() {} }\nimpl Foo<u16> { fn new() {} }\nstruct S;\n\
            impl S { fn get(&self) {} }\nimpl Clone for S { fn clone(&self) -> S { S } }\nfn main() {}\n");
        // A third `new` block first, a split inherent block, the trait impl moved up, a new trait impl.
        let edited = ids("struct Foo<T>(T);\nimpl Foo<u32> { fn new() {} }\nimpl S { fn other(&self) {} }\n\
            impl Clone for S { fn clone(&self) -> S { S } }\nimpl Foo<u8> { fn new() {} }\nimpl Foo<u16> { fn new() {} }\n\
            struct S;\nimpl Default for S { fn default() -> S { S } }\nimpl S { fn get(&self) {} }\nfn main() {}\n");
        let names = |v: &[(String, DefPathHash)]| v.iter().map(|(n, _)| n.clone()).collect::<Vec<String>>();
        assert_eq!(names(&base), ["new", "new", "get", "clone"]);
        assert_eq!(names(&edited), ["new", "other", "clone", "new", "new", "default", "get"]);
        assert_ne!(base[0].1, base[1].1, "Foo<u8>::new and Foo<u16>::new stay apart");
        assert_eq!(base[0].1, edited[3].1, "Foo<u8>::new");
        assert_eq!(base[1].1, edited[4].1, "Foo<u16>::new");
        assert_eq!(base[2].1, edited[6].1, "S::get");
        assert_eq!(base[3].1, edited[2].1, "<S as Clone>::clone");
        assert_ne!(edited[0].1, edited[3].1, "Foo<u32>::new is its own");
    }

    #[test]
    fn method_paths_use_the_parent_impl() {
        let k = krate();
        let (foo, foo_t) = impl_path(k, &[], "Foo", 0);
        let (foo2, foo2_t) = impl_path(k, &[], "Foo", 1);
        let (bar, bar_t) = impl_path(k, &[], "Bar", 0);
        let m = method_path(foo, &foo_t, "get").0;
        assert_eq!(m, method_path(foo, &foo_t, "get").0);
        assert_ne!(m, method_path(foo2, &foo2_t, "get").0, "impl ordinal");
        assert_ne!(m, method_path(bar, &bar_t, "get").0, "impl type");
        assert_ne!(m, method_path(foo, &foo_t, "set").0, "method name");
        assert_ne!(m, free_fn_path(k, &[], "get").0, "free fn of the same name");
    }

    /// The hash-only forms must agree with the path forms: `collect` hashes
    /// without text on every compile, the driver builds text for slot keys,
    /// and both must name the same function.
    #[test]
    fn hash_only_forms_match_path_forms() {
        let k = krate();
        assert_eq!(free_fn_hash(k, &["a"], "run"), free_fn_path(k, &["a"], "run").0);
        let (ih, it) = impl_path(k, &[], "Foo", 2);
        assert_eq!(impl_hash(k, &[], "Foo", 2), ih);
        assert_eq!(method_hash(ih, "get"), method_path(ih, &it, "get").0);
        assert_eq!(child_hash(ih, DefKind::Method, "get", 0), child_path(ih, &it, DefKind::Method, "get", 0).0);
    }

    /// Sequential generated names must not collide: the old FxHash-based
    /// identity made `filler_19` and `filler_92` hash equal, so a valid
    /// file of numbered functions failed with an internal collision
    /// error. A few thousand names from each common generated family
    /// must all hash apart.
    #[test]
    fn sequential_names_do_not_collide() {
        let k = krate();
        let mut seen = std::collections::HashSet::new();
        for i in 0..2000 {
            for name in [format!("filler_{i}"), format!("test_{i}"), format!("a{i}"), format!("fn_{i}"), format!("m{i}_x")] {
                assert!(seen.insert(free_fn_path(k, &[], &name).0), "collision at {name}");
            }
        }
    }

    /// A duplicate definition (same path) is not a collision. Two distinct
    /// paths forced onto one hash are, and the error names both.
    #[test]
    fn collision_check_names_both_paths() {
        let mut paths: FxMap<DiskDefId, Box<str>> = FxMap::default();
        let h = DiskDefId { krate: StableCrateHash(1), hash: DefPathHash(42) };
        insert_path(&mut paths, h, "fn:main#0").unwrap();
        insert_path(&mut paths, h, "fn:main#0").unwrap();
        let other_crate = DiskDefId { krate: StableCrateHash(2), hash: DefPathHash(42) };
        insert_path(&mut paths, other_crate, "fn:other#0").unwrap();
        let err = insert_path(&mut paths, h, "fn:other#0").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("DefPathHash collision"), "{msg}");
        assert!(msg.contains("fn:main#0") && msg.contains("fn:other#0"), "{msg}");
    }

    /// The disk pair is (stable crate, path hash), 16 bytes, and round-trips.
    /// A truncated form never reads as a value.
    #[test]
    fn disk_pair_round_trips() {
        let d = DiskDefId { krate: StableCrateHash(0x0123_4567_89ab_cdef), hash: DefPathHash(7) };
        assert_eq!(DiskDefId::from_bytes(&d.to_bytes()), Some(d));
        assert_eq!(DiskDefId::from_bytes(&d.to_bytes()[..15]), None);
        assert_eq!(DiskDefId::from_bytes(&[]), None);
    }
}
