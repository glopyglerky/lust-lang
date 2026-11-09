// The `Defs` table skeleton (docs/ARCH.md section 2.2, serial refactor S3a).
//
// One table per crate maps each `DefId` to its definition record: kind, parent,
// name, generics, and a per-kind signature payload with resolved types only (a
// signature may hold `Param`, `Projection`, and `Opaque`, never `Infer`). Today the
// payloads that already exist stay where they are behind compat queries: struct and
// enum fields live in `check::collect::AdtDef` (reached by `local_adt`), and fn
// params and returns in `FnInfo`. `Defs` owns identity, kind, parent, generics, and
// param envs now; W-TRAITS-CORE moves payloads in as generic collect lands. All
// signature reads that would walk raw predicate lists for supertraits go through
// `param_env` instead (ARCH 2.1).
use super::*;
use crate::check::spec;
use crate::def::DefId;
use crate::diag::Span;
use crate::intern::{FxMap, Sym};

/// One module in a crate's module tree. Today a crate is one file with one root
/// module; W-RESOLVE fills the tree (ARCH 1.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ModuleId(pub u32);

/// The crate root module, the only module today.
pub const ROOT_MODULE: ModuleId = ModuleId(0);

/// The scope of spliced std source (`impl<T> Option<T>` in library/core,
/// W-STD-A): no user item or import is visible from it, so a user's
/// own `Option` or `Some` cannot capture std's names. Names resolve to
/// builtins and the lang items only.
pub const STD_MODULE: ModuleId = ModuleId(u32::MAX - 1);

/// Visibility of a def. The restricted forms (`pub(crate)`, `pub(in path)`) get
/// one case here; ARCH 1.5 owns their meaning.
#[allow(dead_code)] // Restricted forms are recorded when modules land (W-RESOLVE).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vis {
    Pub,
    Restricted,
    Priv,
}

/// What a def is. This is the definition-table kind of ARCH 2.2, not the def-path
/// hash tag of the same name in `crate::def`; the hash tag stays frozen there and
/// the two never convert silently. New variants go at the end only.
/// `TraitAlias` is absent on purpose: it is a reject in the subset (ARCH 2.2).
#[allow(dead_code)] // Kinds for items not yet in the subset are recorded by their feature waves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DefKind {
    Module,
    Struct,
    Enum,
    Union,
    Variant,
    Field,
    Trait,
    AssocTy,
    AssocConst,
    AssocFn,
    Fn,
    Method,
    Const,
    Static,
    TypeAlias,
    Macro,
    Impl,
    Closure,
    Opaque,
    ForeignFn,
    ForeignStatic,
}

/// The per-kind signature payload of a def, with resolved types only. Adt fields
/// and variant payloads stay in the compat `AdtDef` today (see the file header),
/// so only the fn shape carries types here yet.
#[allow(dead_code)] // Read when payloads move out of the compat tables (W-TRAITS-CORE).
#[derive(Clone, Debug)]
pub enum SigData {
    /// No payload yet, or the payload lives in a compat table.
    Empty,
    /// A fn, method, or assoc fn signature. `params` includes `self` first.
    Fn { params: Vec<Ty>, ret: Ty },
}

/// One definition record (ARCH 2.2 `DefData`).
#[allow(dead_code)] // Name, module, vis, and span are read by resolution and diagnostics in later waves.
#[derive(Clone, Debug)]
pub struct DefData {
    pub kind: DefKind,
    pub parent: Option<DefId>,
    pub name: Sym,
    pub module: ModuleId,
    pub vis: Vis,
    pub generics: Generics,
    pub span: Span,
    pub sig: SigData,
}

impl DefData {
    /// A record with empty generics and no payload, in the root module. Collect
    /// fills the payload compat tables separately today.
    pub fn skeleton(kind: DefKind, parent: Option<DefId>, name: Sym, vis: Vis, span: Span) -> DefData {
        DefData { kind, parent, name, module: ROOT_MODULE, vis, generics: Generics::default(), span, sig: SigData::Empty }
    }
}

/// The definition table of one crate, plus the compat bridge between `DefId` and
/// the session-local `AdtId` index, and the interned param envs. Built by collect;
/// after it, the table is read-only apart from param-env memoization through
/// interior queries in later waves (today envs are interned eagerly at insert).
#[derive(Default)]
pub struct Defs {
    data: Vec<(DefId, DefData)>,
    by_def: FxMap<DefId, u32>,
    adt_locals: FxMap<DefId, AdtId>,
    adt_defs: Vec<DefId>,
    #[allow(dead_code)] // Read through `param_env_data`; production readers land with traits.
    envs: Vec<ParamEnvData>,
    #[allow(dead_code)] // See `envs`.
    env_by_data: FxMap<ParamEnvData, ParamEnvId>,
    #[allow(dead_code)] // See `envs`.
    env_of_def: FxMap<DefId, ParamEnvId>,
}

impl Defs {
    /// An empty table. The empty param env is interned on first use.
    pub fn new() -> Defs {
        Defs::default()
    }

    /// Visits every span the table holds: each def's span and its
    /// generic parameters' spans (check/prog_shift.rs moves them).
    pub fn for_each_span(&mut self, f: &mut dyn FnMut(&mut Span)) {
        for (_, d) in &mut self.data {
            let DefData { kind: _, parent: _, name: _, module: _, vis: _, generics, span, sig: _ } = d;
            f(span);
            generics.params.iter_mut().for_each(|p| f(&mut p.span));
        }
    }

    /// Records one def. A second record for the same `DefId` replaces the first;
    /// duplicate definitions are user errors reported by collect (L0106), and the
    /// surviving record only feeds later error messages.
    pub fn insert(&mut self, def: DefId, data: DefData) {
        spec::wrote(spec::tab::DEFS, &def);
        match self.by_def.get(&def) {
            Some(&i) => self.data[i as usize].1 = data,
            None => {
                self.by_def.insert(def, self.data.len() as u32);
                self.data.push((def, data));
            }
        }
    }

    /// The number of def records.
    pub fn len(&self) -> usize {
        spec::read_all(spec::tab::DEFS);
        self.data.len()
    }

    /// Drops the records added after the first `n` (check/const_body.rs
    /// removes the defs of its transient fns). Adt records stay below `n`.
    pub fn truncate(&mut self, n: usize) {
        spec::wrote_all(spec::tab::DEFS);
        for (def, _) in self.data.drain(n.min(self.data.len())..) {
            self.by_def.remove(&def);
        }
    }

    /// Records one user Adt def and its session-local index, in both directions.
    pub fn insert_adt(&mut self, def: DefId, local: AdtId, data: DefData) {
        spec::wrote(spec::tab::DEF_ADT_LOCALS, &def);
        self.adt_locals.insert(def, local);
        let i = local.0 as usize;
        if i >= self.adt_defs.len() {
            self.adt_defs.resize(i + 1, DefId::PLACEHOLDER);
        }
        spec::wrote(spec::tab::DEF_ADT_DEFS, &i);
        self.adt_defs[i] = def;
        self.insert(def, data);
    }

    /// The record of a def, if this crate defines it.
    pub fn get(&self, def: DefId) -> Option<&DefData> {
        spec::read(spec::tab::DEFS, &def);
        self.by_def.get(&def).map(|&i| &self.data[i as usize].1)
    }

    /// The record of a def for an update in place (collect's
    /// signature normalization keeps `sig` in step with `FnInfo`).
    pub fn get_mut(&mut self, def: DefId) -> Option<&mut DefData> {
        spec::wrote(spec::tab::DEFS, &def);
        self.by_def.get(&def).map(|&i| &mut self.data[i as usize].1)
    }
    /// The kind of a def, if known.
    #[allow(dead_code)] // First production readers are the trait and mono waves.
    pub fn kind_of(&self, def: DefId) -> Option<DefKind> {
        self.get(def).map(|d| d.kind)
    }
    /// The parent def (impl or trait for a method, Adt for a variant), if any.
    #[allow(dead_code)] // See `kind_of`.
    pub fn parent_of(&self, def: DefId) -> Option<DefId> {
        self.get(def).and_then(|d| d.parent)
    }
    /// The generics a def owns: params and written predicates.
    #[allow(dead_code)] // First production readers are generic collect and mono.
    pub fn generics_of(&self, def: DefId) -> Option<&Generics> {
        self.get(def).map(|d| &d.generics)
    }
    /// Records one materialized Adt instance's local index under
    /// its template def. The def-to-template map is left alone:
    /// `local_adt` still answers the template.
    pub fn insert_adt_instance(&mut self, def: DefId, local: AdtId) {
        let i = local.0 as usize;
        if i >= self.adt_defs.len() {
            self.adt_defs.resize(i + 1, DefId::PLACEHOLDER);
        }
        spec::wrote(spec::tab::DEF_ADT_DEFS, &i);
        self.adt_defs[i] = def;
    }

    /// The session-local index of a user Adt def, for the compat `AdtDef` table.
    pub fn local_adt(&self, def: DefId) -> Option<AdtId> {
        spec::read(spec::tab::DEF_ADT_LOCALS, &def);
        self.adt_locals.get(&def).copied()
    }
    /// The def of a user Adt by its session-local index.
    pub fn def_of_adt(&self, local: AdtId) -> Option<DefId> {
        spec::read(spec::tab::DEF_ADT_DEFS, &(local.0 as usize));
        self.adt_defs.get(local.0 as usize).copied().filter(|d| *d != DefId::PLACEHOLDER)
    }

    /// Interns one elaborated param env. The same facts always get the same id.
    #[allow(dead_code)] // Reached through `param_env` today; direct callers land with traits.
    pub fn intern_param_env(&mut self, data: ParamEnvData) -> ParamEnvId {
        if let Some(&id) = self.env_by_data.get(&data) {
            return id;
        }
        let id = ParamEnvId(self.envs.len() as u32);
        self.env_by_data.insert(data.clone(), id);
        self.envs.push(data);
        id
    }

    /// The elaborated param env of a def: its written predicates closed under
    /// supertraits and associated equalities (ARCH 2.1). Computed on first ask and
    /// memoized per def. `supertraits` is the same provider `elaborate` takes;
    /// production passes a closure over this table's trait records, which are all
    /// empty today, so every env is the empty env until traits collect.
    #[allow(dead_code)] // First production callers are selection and normalization (W-TRAITS-CORE).
    pub fn param_env(&mut self, store: &impl TyIntern, def: DefId, supertraits: &dyn Fn(&TraitRef) -> Vec<Predicate>) -> ParamEnvId {
        if let Some(&id) = self.env_of_def.get(&def) {
            return id;
        }
        let written: Vec<Predicate> = self.get(def).map_or_else(Vec::new, |d| d.generics.predicates.clone());
        let facts = elaborate(store, &written, supertraits);
        let id = self.intern_param_env(ParamEnvData { predicates: facts.into() });
        self.env_of_def.insert(def, id);
        id
    }

    /// The facts of an interned param env, in elaboration (first occurrence) order.
    #[allow(dead_code)] // See `param_env`.
    pub fn param_env_data(&self, id: ParamEnvId) -> &ParamEnvData {
        &self.envs[id.0 as usize]
    }
}
