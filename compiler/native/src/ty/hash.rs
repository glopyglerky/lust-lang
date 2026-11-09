// The hashed collections lust knows by def (W-STD-B): `HashMap`,
// `HashSet`, and the std hasher types. Each is a builtin Adt with its
// own def; values live in C over the hashbrown-exact table core
// (src/runtime/hash.h), with the element layout and callbacks cgen
// generates per instance (cgen/hash.rs).
use super::*;

/// One hashed-collection struct lust knows by def.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HashTy {
    /// `HashMap<K, V, S>`: args `[K, V, S]`.
    HashMap,
    /// `HashSet<T, S>`: args `[T, S]`.
    HashSet,
    /// `hash::RandomState`, the default `S`.
    RandomState,
    /// `hash::DefaultHasher` (SipHash-1-3), RandomState's hasher.
    DefaultHasher,
    /// `hash::BuildHasherDefault<H>`: args `[H]`.
    BuildHasherDefault,
    /// `hash_map::Entry<'a, K, V>`: args `[K, V]`. Its methods, and a
    /// match on its two variants (`PatRes::Entry`), whose payloads are
    /// the Entry's C struct read as an `OccupiedEntry` or `VacantEntry`.
    Entry,
    /// `hash_map::OccupiedEntry<'a, K, V>`: args `[K, V]`, made by
    /// `Entry::insert_entry`. Its C form is the Entry's (the slot is set,
    /// the key field unused).
    OccupiedEntry,
    /// `hash_map::VacantEntry<'a, K, V>`: args `[K, V]`, the payload of
    /// `Entry::Vacant`. Its C form is the Entry's (no slot yet, the probe
    /// key owned until `insert`).
    VacantEntry,
    // The C forms of the hash iterators (`IterKind::is_hash`), never
    // types: no def, not in `ALL`. Args are `[K, V]` for a map's,
    // `[T]` for a set's; the variant says which part an item is.
    /// Borrowing, item `(&K, &V)` (`iter`, `iter_mut`).
    IterPairs,
    /// Borrowing, item `&K` (`keys`, a set's `iter`).
    IterKeys,
    /// Borrowing, item `&V` (`values`, `values_mut`).
    IterValues,
    /// Owning, item `(K, V)` (`into_iter`).
    IntoPairs,
    /// Owning, item `K` (`into_keys`, a set's `into_iter`).
    IntoKeys,
    /// Owning, item `V` (`into_values`).
    IntoValues,
    /// A map's `drain` (R20): item `(K, V)`; holds the map's table and a
    /// pointer back to the map's slot for it.
    DrainPairs,
    /// A set's `drain`: item `T`.
    DrainKeys,
}

impl HashTy {
    pub const ALL: [HashTy; 8] = [
        HashTy::HashMap,
        HashTy::HashSet,
        HashTy::RandomState,
        HashTy::DefaultHasher,
        HashTy::BuildHasherDefault,
        HashTy::Entry,
        HashTy::OccupiedEntry,
        HashTy::VacantEntry,
    ];

    /// The std module path and struct name: the def hash input and the
    /// printed name.
    pub fn path(self) -> (&'static [&'static str], &'static str) {
        match self {
            HashTy::HashMap => (&["std", "collections", "hash_map"], "HashMap"),
            HashTy::HashSet => (&["std", "collections", "hash_set"], "HashSet"),
            HashTy::RandomState => (&["std", "hash", "random"], "RandomState"),
            HashTy::DefaultHasher => (&["std", "hash", "random"], "DefaultHasher"),
            HashTy::BuildHasherDefault => (&["core", "hash"], "BuildHasherDefault"),
            HashTy::Entry => (&["std", "collections", "hash_map"], "Entry"),
            HashTy::OccupiedEntry => (&["std", "collections", "hash_map"], "OccupiedEntry"),
            HashTy::VacantEntry => (&["std", "collections", "hash_map"], "VacantEntry"),
            _ => (&["std", "collections", "hash_map"], "Iter"),
        }
    }

    /// The C form of a hash iterator kind.
    pub fn of_iter(k: crate::ty::iter::IterKind) -> HashTy {
        use crate::ty::iter::IterKind as K;
        match k {
            K::MapIter | K::MapIterMut => HashTy::IterPairs,
            K::MapKeys | K::SetIter => HashTy::IterKeys,
            K::MapValues | K::MapValuesMut => HashTy::IterValues,
            K::MapIntoIter => HashTy::IntoPairs,
            K::MapIntoKeys | K::SetIntoIter => HashTy::IntoKeys,
            K::MapDrain => HashTy::DrainPairs,
            K::SetDrain => HashTy::DrainKeys,
            _ => HashTy::IntoValues,
        }
    }

    /// True for the iterator C forms.
    pub fn is_iter_form(self) -> bool {
        self as usize >= HashTy::IterPairs as usize
    }

    /// True for the drain C forms.
    pub fn is_drain(self) -> bool {
        matches!(self, HashTy::DrainPairs | HashTy::DrainKeys)
    }

    /// True for the owning iterator C forms.
    pub fn is_owning_iter(self) -> bool {
        matches!(self, HashTy::IntoPairs | HashTy::IntoKeys | HashTy::IntoValues)
    }

    /// The number of type args.
    pub fn n_args(self) -> usize {
        match self {
            HashTy::HashMap => 3,
            HashTy::HashSet | HashTy::Entry | HashTy::OccupiedEntry | HashTy::VacantEntry => 2,
            HashTy::BuildHasherDefault => 1,
            HashTy::RandomState | HashTy::DefaultHasher => 0,
            _ => 2,
        }
    }

    /// The kind a std path's last segment names.
    pub fn by_name(name: &str) -> Option<HashTy> {
        HashTy::ALL.iter().copied().find(|k| k.path().1 == name)
    }
}

/// The def of one hash struct (see `BuiltinDefs::hash_tys`).
pub(super) fn hash_def(k: HashTy) -> DefId {
    let (module, name) = k.path();
    builtin_def(module, name)
}

/// The hash kind and type args of a type, when it is one of these.
pub fn hash_parts(tys: &impl TyIntern, t: Ty) -> Option<(HashTy, Vec<Ty>)> {
    let Some(BuiltinTy::Hash(k)) = tys.builtin_of(t) else { return None };
    let (_, a) = tys.adt_parts(t)?;
    Some((k, tys.args(a).iter().filter_map(|g| g.as_ty()).collect()))
}

/// The type of one hash struct at `args`.
pub fn mk_hash(tys: &mut impl TyIntern, k: HashTy, args: &[Ty]) -> Ty {
    let def = tys.builtins().hash_tys[k as usize];
    let a: Vec<GenericArg> = args.iter().map(|&t| GenericArg::Ty(t)).collect();
    tys.mk_adt(def, &a)
}

/// The error for a hash type named without its import (rustc E0412):
/// none of these is in the prelude.
pub fn not_imported(k: HashTy, span: crate::diag::Span) -> crate::diag::Diag {
    let name = k.path().1;
    let fix = match k {
        HashTy::HashMap | HashTy::HashSet => format!("Import it: `use std::collections::{name};`."),
        HashTy::Entry => "Name the entry through its methods only (`map.entry(k).or_insert(v)`).".to_string(),
        HashTy::OccupiedEntry | HashTy::VacantEntry => format!("Import it: `use std::collections::hash_map::{name};`."),
        _ => format!("Import it: `use std::hash::{name};`."),
    };
    crate::diag::Diag::new("L0102", span, format!("cannot find type `{name}` in this scope"), fix)
}
