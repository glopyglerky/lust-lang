// The module dependency record (B3 step 2; B3-DESIGN.md 4.2, after the review
// in B3-REVIEW-SOL.md). Per module: hashes of what a full resolve read (the
// module's own items with their sub-items, its imports, its place in the module
// tree) and wrote (its namespaces with the def records of what they bind), and
// the modules its imports read. Per file: its text hash and what its path
// rewrite read (each lookup by module, namespace and name; whole namespaces a
// fn-body glob listed; questions to the crate-wide set of local names). Per
// lookup key: the answer's hash. A full resolve builds one under `LUST_B3_OBSERVE=1`
// (`ResolvedCrate::mdr`); the driver keeps the last one and prints what a warm
// resolve would have had to redo (`observe`).
//
// Step 3 uses the per-file part: every persistent resolve records what each
// file's rewrite read, with each answer's hash (`ChunkDeps`), next to the
// file's cached final chunk. A later resolve takes that chunk again while the
// fixed structure (`structure`) and every recorded answer stand
// (`still_valid`), even when the session-wide rewrite epoch moved.
use crate::diag::SourceMap;
use crate::intern::{FxHasher, FxMap, Interner, Sym};
use crate::resolve::imports::{BindState, Import, ImportKind, ResolvedTables, Target, VisKind};
use crate::ty::defs::ModuleId;
use std::cell::RefCell;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicBool, Ordering};

/// True when `LUST_B3_OBSERVE` is set (and not `0`).
pub fn on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("LUST_B3_OBSERVE").is_ok_and(|v| !v.is_empty() && v != "0"))
}

/// Set while an observed resolve rewrites, so the table reads elsewhere pay
/// one relaxed load only.
static RECORDING: AtomicBool = AtomicBool::new(false);

thread_local! {
    /// The reads of the file this thread rewrites (`begin` .. `end`).
    static READS: RefCell<Option<FileReads>> = const { RefCell::new(None) };
}

/// What one file's path rewrite read.
#[derive(Clone, Default)]
pub struct FileReads {
    /// Modules whose whole namespaces it read (a glob in a fn body lists them).
    pub modules: Vec<u32>,
    /// Each (module, namespace, name) it looked up, found or not.
    pub keys: Vec<(u32, u8, Sym)>,
    /// Names it asked the crate-wide set of local names about.
    pub names: Vec<Sym>,
}

/// Turns recording on or off for the rewrite pass of an observed resolve.
pub fn recording(on: bool) {
    RECORDING.store(on, Ordering::Relaxed);
}

/// Starts recording the reads of one file's rewrite on this thread.
pub fn begin() {
    if RECORDING.load(Ordering::Relaxed) {
        READS.with(|r| *r.borrow_mut() = Some(FileReads::default()));
    }
}

/// Ends `begin`: the reads, each module and name once.
pub fn end() -> Option<FileReads> {
    let mut r = READS.with(|r| r.borrow_mut().take())?;
    r.modules.sort_unstable();
    r.modules.dedup();
    r.keys.sort_unstable();
    r.keys.dedup();
    r.names.sort_unstable();
    r.names.dedup();
    Some(r)
}

/// Notes a lookup of `name` in namespace `ns` of module `m` by the file this
/// thread rewrites.
#[inline]
pub fn read_key(m: ModuleId, ns: u8, name: Sym) {
    if RECORDING.load(Ordering::Relaxed) {
        READS.with(|r| {
            if let Some(r) = r.borrow_mut().as_mut() {
                r.keys.push((m.0, ns, name));
            }
        });
    }
}

/// Notes a read of module `m`'s whole namespaces by the file this thread
/// rewrites.
#[inline]
pub fn read(m: ModuleId) {
    if RECORDING.load(Ordering::Relaxed) {
        READS.with(|r| {
            if let Some(r) = r.borrow_mut().as_mut() {
                r.modules.push(m.0);
            }
        });
    }
}

/// Notes a question to the crate-wide set of local names about `name`.
#[inline]
pub fn read_name(name: Sym) {
    if RECORDING.load(Ordering::Relaxed) {
        READS.with(|r| {
            if let Some(r) = r.borrow_mut().as_mut() {
                r.names.push(name);
            }
        });
    }
}

/// One resolve's record. Modules and files are in id order; the keys match
/// them across resolves.
#[derive(Clone, Default)]
pub struct Mdr {
    pub modules: Vec<ModRec>,
    pub files: Vec<FileRec>,
    /// Inputs of the whole crate: extern crates and the extern prelude, the
    /// crate cfgs, the std modules, platform and globs, `$crate` roots.
    pub global: u64,
    /// The macro environment: every file's textual macro scope and every macro
    /// namespace binding, so a changed `use` of a macro counts (review issue 4).
    pub macros: u64,
    /// The crate-wide set of local names the rewrite asks about.
    pub local_names: FxMap<Sym, u8>,
    /// Each (module, namespace, name) answer's hash: the binding with the def
    /// record of its target, and the key's std alias, glob source and
    /// ambiguity entries.
    pub answers: FxMap<(u32, u8, Sym), u64>,
    /// Files this resolve rewrote (the rest took a cached final chunk).
    pub rewritten: usize,
}

/// One module's record.
#[derive(Clone, Default)]
pub struct ModRec {
    /// The crate root's id and the module path: the key across resolves.
    pub key: String,
    /// Its own items: name, kind, visibility and def hash of each def it
    /// defines, with variants, field visibility, the associated item
    /// visibility its impls give (impls in fn bodies included), and its
    /// `macro_rules!` definitions.
    pub items: u64,
    /// Its imports: kind, path, visibility and rename (no spans).
    pub uses: u64,
    /// Its parent, name, visibility, flags, and its child modules in order.
    pub tree: u64,
    /// What resolving wrote for it: its three namespaces (ambiguity included)
    /// with the def records of the defs they bind (visibility, variants,
    /// associated item and field visibility), and its side tables.
    pub out: u64,
    /// Modules whose namespaces its imports read: each module a glob or
    /// single import's path walked.
    pub sources: Vec<u32>,
}

/// One file's record.
#[derive(Clone, Default)]
pub struct FileRec {
    /// The cache key: the canonical path and the module.
    pub key: String,
    pub text: u64,
    /// None when unknown (a cached final chunk stored by an unobserved resolve).
    pub reads: Option<FileReads>,
}

/// What one crate's resolve adds to the record (`build`).
pub struct CratePart {
    /// Per module (id), the hash of its imports (`imports_part`).
    pub uses: Vec<(u32, u64)>,
    /// (source, importer): the modules each module's imports read.
    pub edges: Vec<(u32, u32)>,
    pub files: Vec<FileRec>,
    /// Files rewritten (not taken from the final chunk cache).
    pub rewritten: usize,
}

fn hash_vis(h: &mut FxHasher, v: &VisKind) {
    match v {
        VisKind::Priv => h.write_u8(1),
        VisKind::Pub => h.write_u8(2),
        VisKind::Crate => h.write_u8(3),
        VisKind::Super => h.write_u8(4),
        VisKind::In(path) => {
            h.write_u8(5);
            path.hash(h);
        }
    }
}

fn hash_target(h: &mut FxHasher, t: &Target) {
    match t {
        Target::Def(d) => (1u8, d.hash.0, d.krate.0).hash(h),
        Target::Module(m) => (2u8, m.0).hash(h),
        Target::Extern(i) => (3u8, *i).hash(h),
        Target::Variant(d, s) => (4u8, d.hash.0, d.krate.0, s.0).hash(h),
    }
}

/// An order-independent sum of entry hashes.
#[derive(Default)]
struct Sum(u64, u64);

impl Sum {
    fn add(&mut self, f: impl FnOnce(&mut FxHasher)) {
        let mut h = FxHasher::default();
        f(&mut h);
        self.0 = self.0.wrapping_add(h.finish().wrapping_mul(0x9e37_79b9_7f4a_7c15));
        self.1 += 1;
    }
    fn get(&self) -> u64 {
        let mut h = FxHasher::default();
        (self.0, self.1).hash(&mut h);
        h.finish()
    }
}

/// A side table entry of key `k`, in module output sums.
fn side_keyed(out: &mut Vec<Sum>, k: &(ModuleId, u8, Sym), f: &dyn Fn(&mut FxHasher)) {
    if let Some(s) = out.get_mut(k.0.0 as usize) {
        s.add(f);
    }
}

/// Hashes the def record a binding carries along: visibility, variants,
/// associated item and field visibility.
fn def_record(t: &ResolvedTables, h: &mut FxHasher, d: &crate::def::DefId) {
    if let Some(info) = t.def_info.get(d) {
        (info.kind as u8, info.flat.0, info.module.0).hash(h);
        hash_vis(h, &info.vis);
    }
    t.variants.get(d).hash(h);
    if let Some(m) = t.assoc_vis.get(d) {
        let mut s = Sum::default();
        for (name, (v, pm)) in m {
            s.add(|h| {
                name.hash(h);
                hash_vis(h, v);
                pm.0.hash(h);
            });
        }
        s.get().hash(h);
    }
    if let Some((m, fields)) = t.field_vis.get(d) {
        let mut s = Sum::default();
        for (name, v) in fields {
            s.add(|h| {
                name.hash(h);
                hash_vis(h, v);
            });
        }
        (m.0, s.get()).hash(h);
    }
    if let Some((m, vs)) = t.tuple_field_vis.get(d) {
        m.0.hash(h);
        for v in vs {
            hash_vis(h, v);
        }
    }
}

/// What final chunk reuse by reads (B3 step 3) takes as fixed: every module
/// (positional ids, parents, visibility, flags), the module defs, the std
/// globs, extern crates and cfgs, `$crate` roots, std modules, platform and
/// splice start, the macro environment (positions left out, as in `build`),
/// and the crate's files in order. A change in any of them means no reuse by
/// reads at all.
pub fn structure(t: &ResolvedTables, files: &[&str]) -> u64 {
    let mut h = FxHasher::default();
    for m in &t.modules {
        (m.parent.map(|p| p.0), m.name.0, &m.path, m.no_prelude, m.krate_root.0, m.macro_use, m.span.file).hash(&mut h);
        hash_vis(&mut h, &m.vis);
    }
    for d in &t.module_defs {
        (d.hash.0, d.krate.0).hash(&mut h);
    }
    for (m, path) in &t.std_globs {
        (m.0, path).hash(&mut h);
    }
    for e in &t.externs {
        (&e.name, e.def.hash.0, e.def.krate.0, e.is_std_lib, e.owner.0, e.root_module.map(|m| m.0), &e.unsupported).hash(&mut h);
    }
    // Hash maps: order-independent sums.
    let mut sum = Sum::default();
    for (m, (cfg, name)) in &t.crate_cfgs {
        sum.add(|x| (0x01u8, m.0, format!("{cfg:?}"), name).hash(x));
    }
    for (s, m) in &t.crate_magic {
        sum.add(|x| (0x02u8, s.0, m.0).hash(x));
    }
    for (f, (m, scope)) in &t.macro_scopes {
        sum.add(|x| {
            (0x03u8, f.0, m.0).hash(x);
            for (name, d) in &scope.inherited {
                (name.0, d.map(|d| d.hash.0)).hash(x);
            }
            // Without positions (see `build`).
            for e in &scope.events {
                (&e.container, &e.skip_inside).hash(x);
                for (name, d) in &e.macros {
                    (name.0, d.map(|d| d.hash.0)).hash(x);
                }
            }
        });
    }
    for (i, maps) in t.maps.iter().enumerate() {
        for (name, st) in &maps[2] {
            sum.add(|x| {
                (0x04u8, i, name.0).hash(x);
                match st {
                    BindState::Ambiguous => x.write_u8(0),
                    BindState::One(b) => {
                        hash_target(x, &b.target);
                        hash_vis(x, &b.vis);
                        (b.rank, b.def_module.0).hash(x);
                    }
                }
            });
        }
    }
    sum.get().hash(&mut h);
    (t.std_modules.iter().map(|(s, m)| (s.0, m.0)).collect::<Vec<_>>(), t.std_platform.map(|(m, s)| (m.0, s.0)), t.std_splice_lo).hash(&mut h);
    t.alloc_roots.iter().map(|m| m.0).collect::<Vec<_>>().hash(&mut h);
    files.hash(&mut h);
    h.finish()
}

/// One resolve's answers, each computed once (`deps_of`, `still_valid`,
/// `build`): a glob-imported type is bound in hundreds of modules, so its
/// def record hashes once too.
pub struct Answers<'t> {
    t: &'t ResolvedTables,
    keys: FxMap<(u32, u8, Sym), u64>,
    modules: FxMap<u32, u64>,
    defs: FxMap<crate::def::DefId, u64>,
    /// Per module, its std alias, glob source, ambiguity and order entries
    /// (built on the first whole-namespace read).
    sides: Option<FxMap<u32, Sum>>,
}

impl<'t> Answers<'t> {
    pub fn new(t: &'t ResolvedTables) -> Answers<'t> {
        Answers { t, keys: FxMap::default(), modules: FxMap::default(), defs: FxMap::default(), sides: None }
    }

    /// The hash of one def's record (`def_record`).
    fn def(&mut self, d: crate::def::DefId) -> u64 {
        let t = self.t;
        *self.defs.entry(d).or_insert_with(|| {
            let mut h = FxHasher::default();
            def_record(t, &mut h, &d);
            h.finish()
        })
    }

    /// Hashes one binding (or its absence) with its target's def record.
    fn state(&mut self, h: &mut FxHasher, state: Option<&BindState>) {
        match state {
            None => h.write_u8(2),
            Some(BindState::Ambiguous) => h.write_u8(0),
            Some(BindState::One(b)) => {
                h.write_u8(1);
                hash_target(h, &b.target);
                hash_vis(h, &b.vis);
                (b.rank, b.def_module.0).hash(h);
                if let Target::Def(d) | Target::Variant(d, _) = &b.target {
                    self.def(*d).hash(h);
                }
            }
        }
    }

    /// Computes on the pool the key answers that the reads of `deps` (with
    /// structure hash `structure`) need and this memo lacks: each answer is a
    /// pure function of the frozen tables, and a full resolve's chunk checks
    /// (`still_valid`) read hundreds of thousands of keys, tens of thousands
    /// of them distinct. Each run of keys has a memo of its own.
    pub fn prefill<'d>(&mut self, deps: impl Iterator<Item = &'d ChunkDeps>, structure: u64) {
        let mut keys: Vec<(u32, u8, Sym)> =
            deps.filter(|d| d.structure == structure).flat_map(|d| d.reads.keys.iter().copied()).filter(|k| !self.keys.contains_key(k)).collect();
        keys.sort_unstable();
        keys.dedup();
        let t = self.t;
        let runs: Vec<&[(u32, u8, Sym)]> = keys.chunks(512).collect();
        for part in crate::pool::par_map(&runs, |run| {
            let mut a = Answers::new(t);
            run.iter().map(|&k| (k, a.key(k))).collect::<Vec<_>>()
        }) {
            self.keys.extend(part);
        }
    }

    /// The hash of one (module, namespace, name) answer: the binding (or its
    /// absence) with the def record of its target, and the key's ambiguity,
    /// glob source, std alias and order entries. A rewrite that looked the
    /// key up reads no more of it.
    pub fn key(&mut self, k: (u32, u8, Sym)) -> u64 {
        if let Some(&h) = self.keys.get(&k) {
            return h;
        }
        let t = self.t;
        let (m, ns, name) = k;
        let mut h = FxHasher::default();
        self.state(&mut h, t.maps.get(m as usize).and_then(|maps| maps.get(ns as usize)).and_then(|n| n.get(&name)));
        let sk = (ModuleId(m), ns, name);
        if let Some(v) = t.ambig_vis.get(&sk) {
            (0x51u8, format!("{v:?}")).hash(&mut h);
        }
        if let Some(src) = t.glob_src.get(&sk) {
            (0x52u8, src.0).hash(&mut h);
        }
        if let Some((v, path)) = t.std_aliases.get(&sk) {
            (0x53u8, path).hash(&mut h);
            hash_vis(&mut h, v);
        }
        if let Some(src) = t.std_glob_aliases.get(&sk) {
            (0x54u8, src.0).hash(&mut h);
        }
        if let Some(set) = t.order_dep.get(&sk) {
            (0x55u8, format!("{set:?}")).hash(&mut h);
        }
        let out = h.finish();
        self.keys.insert(k, out);
        out
    }

    /// The hash of module `m`'s three namespaces and its std alias, glob
    /// source, ambiguity and order entries, for a read of a whole one (a
    /// fn-body glob lists the bindings and the std aliases).
    pub fn module(&mut self, m: u32) -> u64 {
        if let Some(&h) = self.modules.get(&m) {
            return h;
        }
        let t = self.t;
        let sides = self.sides.get_or_insert_with(|| {
            let mut sides: FxMap<u32, Sum> = FxMap::default();
            for (k, v) in &t.ambig_vis {
                sides.entry(k.0.0).or_default().add(|h| (0x51u8, k.1, k.2.0, format!("{v:?}")).hash(h));
            }
            for (k, src) in t.glob_src.iter() {
                sides.entry(k.0.0).or_default().add(|h| (0x52u8, k.1, k.2.0, src.0).hash(h));
            }
            for (k, (v, path)) in &t.std_aliases {
                sides.entry(k.0.0).or_default().add(|h| {
                    (0x53u8, k.1, k.2.0, path).hash(h);
                    hash_vis(h, v);
                });
            }
            for (k, src) in &t.std_glob_aliases {
                sides.entry(k.0.0).or_default().add(|h| (0x54u8, k.1, k.2.0, src.0).hash(h));
            }
            for (k, set) in &t.order_dep {
                sides.entry(k.0.0).or_default().add(|h| (0x55u8, k.1, k.2.0, format!("{set:?}")).hash(h));
            }
            sides
        });
        let side = sides.get(&m).map_or(0, Sum::get);
        let mut s = Sum::default();
        s.add(|h| (0x5eu8, side).hash(h));
        for (ns, map) in t.maps.get(m as usize).into_iter().flatten().enumerate() {
            for (name, state) in map {
                let mut h = FxHasher::default();
                (ns, name.0).hash(&mut h);
                self.state(&mut h, Some(state));
                s.add(|x| h.finish().hash(x));
            }
        }
        let out = s.get();
        self.modules.insert(m, out);
        out
    }
}

/// What a cached final chunk's rewrite read, with each answer's hash then
/// (B3 step 3): the chunk stands while `still_valid` holds.
#[derive(Clone, Default)]
pub struct ChunkDeps {
    pub structure: u64,
    pub reads: FileReads,
    /// Parallel to `reads.keys`, `reads.modules` and `reads.names`.
    pub key_answers: Vec<u64>,
    pub module_answers: Vec<u64>,
    pub name_answers: Vec<u8>,
}

/// The dependencies of one file's fresh rewrite.
pub fn deps_of(reads: FileReads, a: &mut Answers, local_names: &FxMap<Sym, u8>, structure: u64) -> ChunkDeps {
    let key_answers = reads.keys.iter().map(|&k| a.key(k)).collect();
    let module_answers = reads.modules.iter().map(|&m| a.module(m)).collect();
    let name_answers = reads.names.iter().map(|s| local_names.get(s).copied().unwrap_or(0)).collect();
    ChunkDeps { structure, reads, key_answers, module_answers, name_answers }
}

/// True when every answer `d` recorded is the same under the current tables.
pub fn still_valid(d: &ChunkDeps, a: &mut Answers, local_names: &FxMap<Sym, u8>, structure: u64) -> bool {
    d.structure == structure
        && d.reads.keys.iter().zip(&d.key_answers).all(|(&k, &h)| a.key(k) == h)
        && d.reads.modules.iter().zip(&d.module_answers).all(|(&m, &h)| a.module(m) == h)
        && d.reads.names.iter().zip(&d.name_answers).all(|(s, &v)| local_names.get(s).copied().unwrap_or(0) == v)
}

/// Per-module import hashes and the modules each module's imports read,
/// from one crate's import list (after `run_imports`).
pub fn imports_part(tables: &ResolvedTables, imports: &[Import], interner: &Interner) -> (Vec<(u32, u64)>, Vec<(u32, u32)>) {
    let mut uses: FxMap<u32, Sum> = FxMap::default();
    let mut edges: Vec<(u32, u32)> = Vec::new();
    for imp in imports {
        let (tag, path, name) = match &imp.kind {
            ImportKind::Single { path, name } => (1u8, path, Some(*name)),
            ImportKind::Glob { path } => (2, path, None),
            ImportKind::CheckPath { path } => (3, path, None),
            ImportKind::BadLeading { path } => (4, path, None),
        };
        uses.entry(imp.module.0).or_default().add(|h| {
            tag.hash(h);
            path.hash(h);
            name.hash(h);
            imp.leading_colon.hash(h);
            hash_vis(h, &imp.vis);
        });
        // Every module the path walks: each prefix that names a module (a
        // glob's whole path too).
        let walk = if tag == 2 { path.len() } else { path.len().saturating_sub(1) };
        for k in 1..=walk {
            if let crate::resolve::imports::PathRes::Done(Target::Module(src)) =
                tables.resolve_module_path(imp.module, &path[..k], imp.leading_colon, interner)
            {
                edges.push((src.0, imp.module.0));
            }
        }
    }
    edges.sort_unstable();
    edges.dedup();
    (uses.into_iter().map(|(m, s)| (m, s.get())).collect(), edges)
}

/// The record of one resolve from its final tables and its crates' parts.
pub fn build(t: &ResolvedTables, map: &SourceMap, parts: Vec<CratePart>) -> Mdr {
    let n = t.modules.len();
    let (mut uses, mut edges, mut files, mut rewritten) = (Vec::new(), Vec::new(), Vec::new(), 0);
    for p in parts {
        uses.extend(p.uses);
        edges.extend(p.edges);
        files.extend(p.files);
        rewritten += p.rewritten;
    }
    let key = |m: usize| {
        let info = &t.modules[m];
        format!("{}#{}", info.krate_root.0, info.path.iter().map(|s| s.0.to_string()).collect::<Vec<_>>().join("::"))
    };
    let mut items: Vec<Sum> = (0..n).map(|_| Sum::default()).collect();
    let mut out: Vec<Sum> = (0..n).map(|_| Sum::default()).collect();
    let mut tree: Vec<Sum> = (0..n).map(|_| Sum::default()).collect();
    for (d, info) in &t.def_info {
        let Some(s) = items.get_mut(info.module.0 as usize) else { continue };
        s.add(|h| {
            (d.hash.0, d.krate.0).hash(h);
            def_record(t, h, d);
        });
    }
    // Associated item visibility belongs to the module of the impl that gives it.
    for (d, m) in &t.assoc_vis {
        for (name, (v, pm)) in m {
            if let Some(s) = items.get_mut(pm.0 as usize) {
                s.add(|h| {
                    (0xa5u8, d.hash.0, name.0).hash(h);
                    hash_vis(h, v);
                });
            }
        }
    }
    for (&(m, name), d) in &t.macro_textual_binds {
        if let Some(s) = items.get_mut(m.0 as usize) {
            s.add(|h| (0x3cu8, name.0, d.hash.0).hash(h));
        }
    }
    let mut children: FxMap<u32, u32> = FxMap::default();
    for (i, info) in t.modules.iter().enumerate() {
        tree[i].add(|h| {
            (info.parent.map(|p| p.0), info.name.0, &info.path, info.no_prelude, info.krate_root.0, info.macro_use).hash(h);
            hash_vis(h, &info.vis);
        });
        // Child order: a swap of two `mod` lines moves textual macro scopes.
        if let Some(p) = info.parent {
            let rank = children.entry(p.0).or_insert(0u32);
            tree[p.0 as usize].add(|h| (0x7eu8, *rank, info.name.0).hash(h));
            *rank += 1;
        }
    }
    for (i, maps) in t.maps.iter().enumerate() {
        for (ns, m) in maps.iter().enumerate() {
            for (name, state) in m {
                out[i].add(|h| {
                    (ns, name.0).hash(h);
                    match state {
                        BindState::Ambiguous => h.write_u8(0),
                        BindState::One(b) => {
                            h.write_u8(1);
                            hash_target(h, &b.target);
                            hash_vis(h, &b.vis);
                            (b.rank, b.def_module.0).hash(h);
                            if let Target::Def(d) | Target::Variant(d, _) = &b.target {
                                def_record(t, h, d);
                            }
                        }
                    }
                });
            }
        }
    }
    let side = |out: &mut Vec<Sum>, m: ModuleId, f: &dyn Fn(&mut FxHasher)| {
        if let Some(s) = out.get_mut(m.0 as usize) {
            s.add(f);
        }
    };
    let keyed = side_keyed;
    for (k, v) in &t.ambig_vis {
        keyed(&mut out, k, &|h| (0x51u8, k.1, k.2.0, format!("{v:?}")).hash(h));
    }
    for (k, src) in t.glob_src.iter() {
        keyed(&mut out, &k, &|h| (0x52u8, k.1, k.2.0, src.0).hash(h));
    }
    for (k, (v, path)) in &t.std_aliases {
        keyed(&mut out, k, &|h| {
            (0x53u8, k.1, k.2.0, path).hash(h);
            hash_vis(h, v);
        });
    }
    for (k, src) in &t.std_glob_aliases {
        keyed(&mut out, k, &|h| (0x54u8, k.1, k.2.0, src.0).hash(h));
    }
    for (k, set) in &t.order_dep {
        keyed(&mut out, k, &|h| (0x55u8, k.1, k.2.0, format!("{set:?}")).hash(h));
    }
    for (m, path) in &t.std_globs {
        side(&mut out, *m, &|h| (0x56u8, path).hash(h));
    }
    let mut sources: Vec<Vec<u32>> = vec![Vec::new(); n];
    for &(src, m) in &edges {
        if let Some(s) = sources.get_mut(m as usize) {
            s.push(src);
        }
    }
    let uses: FxMap<u32, u64> = uses.iter().copied().collect();
    let modules = (0..n)
        .map(|i| ModRec {
            key: key(i),
            items: items[i].get(),
            uses: uses.get(&(i as u32)).copied().unwrap_or(0),
            tree: tree[i].get(),
            out: out[i].get(),
            sources: std::mem::take(&mut sources[i]),
        })
        .collect();
    let mut global = FxHasher::default();
    format!("{:?}", t.externs).hash(&mut global);
    let mut cfgs: Vec<String> = t.crate_cfgs.iter().map(|(m, c)| format!("{} {c:?}", m.0)).collect();
    cfgs.sort_unstable();
    cfgs.hash(&mut global);
    let mut magic: Vec<(u32, u32)> = t.crate_magic.iter().map(|(s, m)| (s.0, m.0)).collect();
    magic.sort_unstable();
    (magic, &t.std_modules.iter().map(|(s, m)| (s.0, m.0)).collect::<Vec<_>>(), t.std_platform.map(|(m, s)| (m.0, s.0))).hash(&mut global);
    t.alloc_roots.iter().map(|m| m.0).collect::<Vec<_>>().hash(&mut global);
    // Without event positions: they move with any text shift in their own
    // file, and only that file (rewritten anyway) reads them; other files
    // see what it passes on (`inherited`) and the order of `mod`s (`tree`).
    let mut macros = Sum::default();
    for (f, (m, scope)) in &t.macro_scopes {
        let events: Vec<String> = scope.events.iter().map(|e| format!("{:?} {:?} {:?}", e.container, e.skip_inside, e.macros)).collect();
        macros.add(|h| (map.path(*f), m.0, format!("{:?}", scope.inherited), events).hash(h));
    }
    for (i, maps) in t.maps.iter().enumerate() {
        for (name, state) in &maps[2] {
            macros.add(|h| (i, name.0, format!("{state:?}")).hash(h));
        }
    }
    // Every key with a binding or a side entry, and its answer.
    let mut keys: Vec<(u32, u8, Sym)> = Vec::new();
    for (i, maps) in t.maps.iter().enumerate() {
        for (ns, m) in maps.iter().enumerate() {
            keys.extend(m.keys().map(|&name| (i as u32, ns as u8, name)));
        }
    }
    let side_keys = t.ambig_vis.keys().chain(t.std_aliases.keys()).chain(t.std_glob_aliases.keys()).chain(t.order_dep.keys());
    keys.extend(side_keys.map(|k| (k.0.0, k.1, k.2)));
    keys.extend(t.glob_src.iter().map(|(k, _)| (k.0.0, k.1, k.2)));
    keys.sort_unstable();
    keys.dedup();
    let mut a = Answers::new(t);
    let answers = keys.into_iter().map(|k| (k, a.key(k))).collect();
    Mdr { modules, files, global: global.finish(), macros: macros.get(), local_names: t.local_named_set(), answers, rewritten }
}

/// What changed from one resolve to the next, and what a warm resolve would
/// have had to redo (`observe`).
#[derive(Debug, Default)]
pub struct Observation {
    pub files: usize,
    pub files_changed: usize,
    /// Files this resolve rewrote (cache misses).
    pub rewritten: usize,
    pub modules: usize,
    /// Modules whose own inputs changed (items, imports, tree), new ones included.
    pub dirty: usize,
    /// `dirty` closed over the modules whose imports read a module in it.
    pub region: usize,
    /// Modules whose namespaces (or what they bind) changed.
    pub actual: usize,
    /// Of `actual`, the modules `region` did not hold, and the first few keys.
    pub missed: usize,
    pub missed_keys: Vec<String>,
    /// Local names that came or went (or changed namespaces).
    pub local_names_changed: usize,
    /// Unchanged files whose rewrite read no module in `actual` and asked about
    /// no local name that changed: their final chunks could be reused.
    pub reusable: usize,
    /// The same with lookups by name: no looked-up answer changed, and no
    /// whole namespace it read is in `actual`.
    pub reusable_keys: usize,
    /// Answers (module, namespace, name) that came, went or changed.
    pub answers_changed: usize,
    /// Lookups the files' records hold, over all files (their size).
    pub keys: usize,
    /// Why a fixed-structure warm resolve would refuse: the module or file set,
    /// the macro environment, or the crate-wide inputs changed.
    pub fallback: Vec<&'static str>,
}

/// Compares the last resolve's record with this one's.
pub fn observe(prev: &Mdr, cur: &Mdr) -> Observation {
    let mut o = Observation { files: cur.files.len(), modules: cur.modules.len(), rewritten: cur.rewritten, ..Default::default() };
    let old_mods: FxMap<&str, &ModRec> = prev.modules.iter().map(|m| (m.key.as_str(), m)).collect();
    let old_files: FxMap<&str, &FileRec> = prev.files.iter().map(|f| (f.key.as_str(), f)).collect();
    if prev.modules.len() != cur.modules.len() || cur.modules.iter().any(|m| !old_mods.contains_key(m.key.as_str())) {
        o.fallback.push("module set");
    }
    if prev.files.len() != cur.files.len() || cur.files.iter().any(|f| !old_files.contains_key(f.key.as_str())) {
        o.fallback.push("file set");
    }
    if prev.macros != cur.macros {
        o.fallback.push("macro environment");
    }
    if prev.global != cur.global {
        o.fallback.push("crate inputs");
    }
    let n = cur.modules.len();
    let mut dirty = vec![false; n];
    let mut actual = vec![false; n];
    for (i, m) in cur.modules.iter().enumerate() {
        match old_mods.get(m.key.as_str()) {
            Some(p) => {
                dirty[i] = (p.items, p.uses, p.tree) != (m.items, m.uses, m.tree);
                actual[i] = p.out != m.out;
            }
            None => (dirty[i], actual[i]) = (true, true),
        }
    }
    let mut readers: Vec<Vec<u32>> = vec![Vec::new(); n];
    for (i, m) in cur.modules.iter().enumerate() {
        for &s in &m.sources {
            if let Some(r) = readers.get_mut(s as usize) {
                r.push(i as u32);
            }
        }
    }
    let mut region = dirty.clone();
    let mut todo: Vec<usize> = (0..n).filter(|&i| dirty[i]).collect();
    while let Some(i) = todo.pop() {
        for &r in &readers[i] {
            if !region[r as usize] {
                region[r as usize] = true;
                todo.push(r as usize);
            }
        }
    }
    o.dirty = dirty.iter().filter(|&&d| d).count();
    o.region = region.iter().filter(|&&d| d).count();
    o.actual = actual.iter().filter(|&&d| d).count();
    for i in (0..n).filter(|&i| actual[i] && !region[i]) {
        o.missed += 1;
        if o.missed_keys.len() < 4 {
            o.missed_keys.push(cur.modules[i].key.clone());
        }
    }
    let changed_names: FxMap<Sym, ()> = cur
        .local_names
        .iter()
        .filter(|(s, v)| prev.local_names.get(s) != Some(v))
        .chain(prev.local_names.iter().filter(|(s, _)| !cur.local_names.contains_key(s)))
        .map(|(s, _)| (*s, ()))
        .collect();
    o.local_names_changed = changed_names.len();
    let changed_keys: FxMap<(u32, u8, Sym), ()> = cur
        .answers
        .iter()
        .filter(|(k, v)| prev.answers.get(k) != Some(v))
        .map(|(k, _)| (*k, ()))
        .chain(prev.answers.keys().filter(|k| !cur.answers.contains_key(k)).map(|k| (*k, ())))
        .collect();
    o.answers_changed = changed_keys.len();
    o.keys = cur.files.iter().filter_map(|f| f.reads.as_ref()).map(|r| r.keys.len()).sum();
    for f in &cur.files {
        if !old_files.get(f.key.as_str()).is_some_and(|p| p.text == f.text) {
            o.files_changed += 1;
            continue;
        }
        let names_stale = |r: &FileReads| r.names.iter().any(|s| changed_names.contains_key(s));
        let whole_stale = |r: &FileReads| r.modules.iter().any(|&m| actual.get(m as usize).copied().unwrap_or(true));
        let stale = f.reads.as_ref().is_none_or(|r| {
            whole_stale(r) || names_stale(r) || r.keys.iter().any(|&(m, _, _)| actual.get(m as usize).copied().unwrap_or(true))
        });
        if !stale {
            o.reusable += 1;
        }
        let key_stale =
            f.reads.as_ref().is_none_or(|r| whole_stale(r) || names_stale(r) || r.keys.iter().any(|k| changed_keys.contains_key(k)));
        if !key_stale {
            o.reusable_keys += 1;
        }
    }
    o.files_changed += prev.files.iter().filter(|f| !cur.files.iter().any(|c| c.key == f.key)).count();
    o
}

impl Observation {
    /// One summary line (`b3-observe: ...`).
    pub fn line(&self) -> String {
        format!(
            "b3-observe: files={} changed={} rewritten={} reusable={} reusable_keys={} answers_changed={} keys={} modules={} dirty={} region={} actual={} missed={}{} local_names_changed={} fallback={}",
            self.files,
            self.files_changed,
            self.rewritten,
            self.reusable,
            self.reusable_keys,
            self.answers_changed,
            self.keys,
            self.modules,
            self.dirty,
            self.region,
            self.actual,
            self.missed,
            if self.missed_keys.is_empty() { String::new() } else { format!(" ({})", self.missed_keys.join(", ")) },
            self.local_names_changed,
            if self.fallback.is_empty() { "-".to_string() } else { self.fallback.join("+") }
        )
    }
}
