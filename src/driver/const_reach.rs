// Which const fns a const evaluation may run (W-PATCH-MISS2). A const fn
// body edit moves nothing outside the fn's own check unless an evaluator
// runs that body: collect's (const and static values, array lengths,
// discriminants, const generic arguments) or a body check's (`const`
// blocks, array repeat lengths, const generic arguments in a body). Every
// evaluator starts from one of those expressions and runs a const fn only
// through a call that names it, so a closure over names bounds what it can
// reach: start from every identifier in those places
// (`ast_hash::const_idents`), then add the body identifiers of each const
// fn whose name is in the set, until nothing changes. A const fn whose name
// is not in the set runs in no evaluator, so an edit of its body is a plain
// body edit (`patch::apply`). Names, not defs: a collision only costs the
// fast path. A method call names its method too (`Flags::A.bits()` holds
// the identifier `bits`), so const methods with a `self` parameter count
// like any const fn.
//
// A reached const fn may still feed only const and static values: then
// the kept program evaluates every value again and compares
// (`collect::const_values_hold`). `revaluable` says so when the names
// that can carry the fn's result (the fn, the const fns that name one of
// them, and the const and static items whose value names one) meet no
// other evaluator start: no array length, const argument, `const` block
// in a body, discriminant, or const param default names one. (A `const`
// block in a const or static value evaluates with that value.)
//
// The sets live in the patch base and only grow: each patch adds what its
// new nodes and its edited const fns can reach, and never removes what an
// old body could (a superset stays sound). Item values are read once: a
// patch keeps every item outside the edited body, and the items inside it
// keep their text (`patch::same_items`). A full resolve drops the sets with
// the base. The one evaluator start outside these places, a
// `thread_local!` `const { .. }` initializer (its block is a plain fn's
// body), records a `Checked` read; the driver finds those readers by their
// reads.
//
// The header delta asks a sharper question for an inherent method
// (`reaches_inherent`): a path `Q::name` in an item value names the
// associated fn of the type `Q` denotes, so `LazyLock::new` names no
// user `Store::new`. Only three root forms can name an associated fn: a
// path `Q::name`, a `<T>::name` path, and a method call `.name(..)`. A bare
// name (a one-segment path, a field, a binding: `self.end`, `fn new(end:
// i32)`) never does, so it does not count. A `Q::name` counts when its
// qualifier may denote the method's Adt: its name, `Self`, or any name a
// type alias, `use` rename, or associated type has (by name, as
// everything here), but not a name the resolver gave a def of another
// crate (a dependency cannot name a type of a crate that depends on it).
// A method call counts only when the method is const after the edit (the
// delta refuses one that was; the check rejects a const context's call of
// a non-const fn, L0631). A qualifier with generic arguments (`Id::<T>::f`,
// through `type Id<T> = T`) and a `<T>::name` path count as unqualified.
// Identifiers inside a closure body in an item value do not count at all
// (outside `const` blocks): the evaluator never runs a closure
// (`ast_hash::ConstRoots`).
use super::compile::fn_def_at;
use super::session::Front;
use crate::intern::{FxHasher, Sym};
use std::collections::HashSet;
use std::hash::BuildHasherDefault;

type FxSet<T> = HashSet<T, BuildHasherDefault<FxHasher>>;

pub(crate) struct ConstReach {
    /// The closure over every evaluator start.
    names: FxSet<Sym>,
    /// Identifiers in the starts other than item values.
    scalar: FxSet<Sym>,
    /// Each const, static, and associated const item: name and the
    /// identifiers of its value.
    values: Vec<(Option<Sym>, Vec<Sym>)>,
    /// Expression, type, and pattern arena lengths already scanned.
    seen: (usize, usize, usize),
    /// The const fns (`Front::fn_loc` index, name), and whether each one's
    /// body identifiers joined `names`.
    consts: Vec<(usize, Sym)>,
    expanded: Vec<bool>,
    /// `names` but the last segments of qualified paths in item values,
    /// and those paths' qualifiers by name (`reaches_inherent`).
    unqual: FxSet<Sym>,
    quals: crate::intern::FxMap<Sym, Vec<Sym>>,
    /// Names a qualifier may give any type: `Self`, type aliases, `use`
    /// renames, and associated types (source names: a resolver flat name's
    /// last part, `intern::source_name`).
    aliases: FxSet<String>,
}

impl ConstReach {
    /// The sets for the front's current Ast.
    pub(crate) fn build(front: &Front) -> ConstReach {
        // A `const` block in a body runs in a body check; one in a const
        // or static value runs with that value.
        let mut bodies: Vec<(u32, u32)> = front.fn_loc.iter().map(|&(item, sub)| fn_def_at(&front.ast, item, sub).exprs).collect();
        trait_bodies(&front.ast.items, &mut bodies);
        bodies.sort_unstable();
        let roots = super::ast_hash::const_idents(&front.ast, (0, 0, 0), true, Some(&bodies), front.known.self_type);
        let consts: Vec<(usize, Sym)> = front
            .fn_loc
            .iter()
            .enumerate()
            .filter_map(|(i, &(item, sub))| {
                let def = fn_def_at(&front.ast, item, sub);
                def.qual.is_const.then_some((i, def.name))
            })
            .collect();
        let mut quals: crate::intern::FxMap<Sym, Vec<Sym>> = Default::default();
        for &(q, n) in &roots.quals {
            quals.entry(n).or_default().push(q);
        }
        for &n in &roots.scalar_calls {
            quals.entry(n).or_default().push(n);
        }
        let mut r = ConstReach {
            names: FxSet::default(),
            scalar: roots.scalar.iter().chain(&roots.scalar_calls).chain(&roots.scalar_bare).copied().collect(),
            values: roots.values,
            seen: arena_lens(front),
            expanded: vec![false; consts.len()],
            consts,
            unqual: roots.scalar.iter().chain(&roots.value_unqual).copied().collect(),
            quals,
            aliases: aliases(front),
        };
        r.names.extend(r.scalar.iter().copied());
        r.names.extend(r.values.iter().flat_map(|v| v.1.iter().copied()));
        r.close(front);
        r
    }

    /// Adds what a patch can make reachable: identifiers in the appended
    /// nodes, and the new bodies of the edited fns (`Front::fn_loc`
    /// indexes) that are reachable const fns.
    pub(crate) fn update(&mut self, front: &Front, edited: &[usize]) {
        let roots = super::ast_hash::const_idents(&front.ast, self.seen, false, None, front.known.self_type);
        self.scalar.extend(roots.scalar.iter().chain(&roots.scalar_calls).chain(&roots.scalar_bare).copied());
        self.unqual.extend(roots.scalar.iter().copied());
        self.names.extend(roots.scalar.iter().chain(&roots.scalar_calls).chain(&roots.scalar_bare).copied());
        for &n in &roots.scalar_calls {
            self.quals.entry(n).or_default().push(n);
        }
        self.seen = arena_lens(front);
        for (k, (i, _)) in self.consts.iter().enumerate() {
            if edited.contains(i) {
                self.expanded[k] = false;
            }
        }
        self.close(front);
    }

    /// True when an evaluator may run the body of fn `i` (a
    /// `Front::fn_loc` index).
    pub(crate) fn reaches(&self, front: &Front, i: usize) -> bool {
        let (item, sub) = front.fn_loc[i];
        let def = fn_def_at(&front.ast, item, sub);
        self.names.contains(&def.name)
    }

    /// `reaches` for fn `i`, an inherent method of the Adt whose (flat)
    /// name is `adt` (see the module docs): false when every root mention
    /// of its name is `Q::name` with `Q` none of the Adt's name and
    /// `aliases`, or a name the resolver gave another crate's def
    /// (`crate_prefix`: a dependency or shipped crate cannot name a type of
    /// a crate that depends on it, so neither can its aliases). A method
    /// call `.name(..)` (the pair (name, name)) counts only for a method
    /// that may be const: the check rejects a const context's call of a
    /// non-const fn (L0631), so no evaluator runs one, and a method call
    /// is no fn pointer value whose header a value reads.
    pub(crate) fn reaches_inherent(&self, front: &Front, i: usize, adt: &str, may_be_const: bool) -> bool {
        let (item, sub) = front.fn_loc[i];
        let name = fn_def_at(&front.ast, item, sub).name;
        if !self.names.contains(&name) {
            return false;
        }
        let own_crate = crate_prefix(adt);
        let adt = crate::intern::source_name(adt).trim_start_matches("r#");
        let may_be = |q: &Sym| {
            let full = front.interner.get(*q);
            if crate_prefix(full).is_some_and(|c| Some(c) != own_crate) {
                return false;
            }
            let q = crate::intern::source_name(full).trim_start_matches("r#");
            q == adt || self.aliases.contains(q)
        };
        self.unqual.contains(&name) || self.quals.get(&name).is_some_and(|qs| qs.iter().any(|q| may_be(q) || (*q == name && may_be_const)))
    }

    /// Why `reaches_inherent` holds, for `LUST_DERIVE_LOG`: the unqualified
    /// mention, or the qualifiers that may denote the Adt.
    pub(crate) fn inherent_why(&self, front: &Front, i: usize, adt: &str) -> String {
        let (item, sub) = front.fn_loc[i];
        let name = fn_def_at(&front.ast, item, sub).name;
        let adt_s = crate::intern::source_name(adt).trim_start_matches("r#");
        let qs: Vec<String> = self.quals.get(&name).map_or_else(Vec::new, |qs| {
            qs.iter()
                .filter(|qq| {
                    let full = front.interner.get(**qq);
                    let q = crate::intern::source_name(full).trim_start_matches("r#");
                    q == adt_s || self.aliases.contains(q) || **qq == name
                })
                .map(|q| front.interner.get(*q).to_string())
                .collect()
        });
        format!("unqual {}, quals {:?}", self.unqual.contains(&name), qs)
    }

    /// True when the results of fns `fns` (`Front::fn_loc` indexes) can
    /// reach only const and static values (see the module docs).
    pub(crate) fn revaluable(&self, front: &Front, fns: &[usize]) -> bool {
        let idents = &front.chunks[0].scan.body_idents;
        let mut carry: FxSet<Sym> = fns
            .iter()
            .map(|&i| {
                let (item, sub) = front.fn_loc[i];
                fn_def_at(&front.ast, item, sub).name
            })
            .collect();
        loop {
            let before = carry.len();
            for &(i, name) in &self.consts {
                if !carry.contains(&name) && idents.get(i).is_some_and(|ids| ids.iter().any(|s| carry.contains(s))) {
                    carry.insert(name);
                }
            }
            for (name, ids) in &self.values {
                if let Some(n) = name
                    && !carry.contains(n)
                    && ids.iter().any(|s| carry.contains(s))
                {
                    carry.insert(*n);
                }
            }
            if carry.len() == before {
                return carry.iter().all(|s| !self.scalar.contains(s));
            }
        }
    }

    /// Adds the body identifiers of every const fn whose name is in the
    /// set, until nothing changes.
    fn close(&mut self, front: &Front) {
        let idents = &front.chunks[0].scan.body_idents;
        loop {
            let mut grew = false;
            for (k, &(i, name)) in self.consts.iter().enumerate() {
                if !self.expanded[k] && self.names.contains(&name) {
                    self.expanded[k] = true;
                    self.names.extend(idents.get(i).into_iter().flatten().copied());
                    let (item, sub) = front.fn_loc[i];
                    let (unqual, quals) = super::ast_hash::body_quals(&front.ast, fn_def_at(&front.ast, item, sub));
                    self.unqual.extend(unqual);
                    // `Self::f` in a method names its impl's self type's `f`.
                    let own = match front.ast.items.get(item).map(|it| &it.kind) {
                        Some(crate::ast::ItemKind::Impl(imp)) if sub.is_some() => match front.ast.ty(imp.self_ty) {
                            crate::ast::TyExpr::Path(segs) => segs.last().map(|s| s.name),
                            _ => None,
                        },
                        _ => None,
                    };
                    for (q, n) in quals {
                        let q = if q == front.known.self_type { own.unwrap_or(q) } else { q };
                        self.quals.entry(n).or_default().push(q);
                    }
                    grew = true;
                }
            }
            if !grew {
                return;
            }
        }
    }
}

/// The crate part of a resolver flat name, when it has one: a dependency
/// crate's hash segment (`k` and 16 hex digits) or a shipped crate's
/// `__lust_` segment. None for a local name, or a name as written.
fn crate_prefix(flat: &str) -> Option<&str> {
    let (first, _) = flat.split_once('$')?;
    let dep = first.len() == 17 && first.starts_with('k') && first[1..].bytes().all(|b| b.is_ascii_hexdigit());
    (dep || first.starts_with("__lust_")).then_some(first)
}

/// `ConstReach::aliases` for the front's Ast: `Self`, and the names of
/// every type alias, `use` rename, and associated type.
fn aliases(front: &Front) -> FxSet<String> {
    fn tree(t: &crate::ast::UseTree, out: &mut FxSet<Sym>) {
        match &t.kind {
            crate::ast::UseKind::Simple { rename: Some(r) } => {
                out.insert(*r);
            }
            crate::ast::UseKind::Group(ts) => ts.iter().for_each(|t| tree(t, out)),
            _ => {}
        }
    }
    fn items(its: &[crate::ast::Item], out: &mut FxSet<Sym>) {
        use crate::ast::ItemKind;
        for it in its {
            match &it.kind {
                ItemKind::Type(a) => {
                    out.insert(a.name);
                }
                ItemKind::Use(u) => tree(&u.tree, out),
                ItemKind::Impl(imp) => out.extend(imp.types.iter().map(|t| t.name)),
                ItemKind::Trait(t) => out.extend(t.types.iter().map(|t| t.name)),
                ItemKind::Mod(m) => items(m.body.as_deref().unwrap_or(&[]), out),
                _ => {}
            }
        }
    }
    let mut out = FxSet::default();
    out.insert(front.known.self_type);
    items(&front.ast.items, &mut out);
    items(&front.ast.stmt_items, &mut out);
    out.into_iter().map(|s| crate::intern::source_name(front.interner.get(s)).trim_start_matches("r#").to_string()).collect()
}

/// The expression ranges of trait default method bodies (not in
/// `Front::fn_loc`), in nested modules too.
fn trait_bodies(items: &[crate::ast::Item], out: &mut Vec<(u32, u32)>) {
    for it in items {
        match &it.kind {
            crate::ast::ItemKind::Trait(t) => out.extend(t.fns.iter().filter(|f| f.body.is_some()).map(|f| f.exprs)),
            crate::ast::ItemKind::Mod(m) => trait_bodies(m.body.as_deref().unwrap_or(&[]), out),
            _ => {}
        }
    }
}

fn arena_lens(front: &Front) -> (usize, usize, usize) {
    (front.ast.exprs.len(), front.ast.tys.len(), front.ast.pats.len())
}
