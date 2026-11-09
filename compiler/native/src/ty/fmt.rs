// The `core::fmt` types lust knows by def (W-STD-B): the Formatter,
// `Arguments`, and the Debug builders. Each is a builtin Adt with its
// own def and no type args (their lifetimes are erased); values live
// in the C runtime (src/runtime/fmt.h). `fmt::Error` is not here: it is
// an ordinary unit struct Adt (`check::generics::fmt_error_def`), so
// its derives and layout are the plain Adt machinery.
use super::*;

/// One `core::fmt` struct lust knows by def.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FmtTy {
    /// `fmt::Formatter<'a>`, only ever behind `&mut` in user code.
    Formatter,
    /// `fmt::Arguments<'a>`: a `format_args!` value. Display and
    /// Debug format its arguments at each use, as in core.
    Arguments,
    DebugStruct,
    DebugTuple,
    DebugList,
    DebugSet,
    DebugMap,
}

impl FmtTy {
    pub const ALL: [FmtTy; 7] =
        [FmtTy::Formatter, FmtTy::Arguments, FmtTy::DebugStruct, FmtTy::DebugTuple, FmtTy::DebugList, FmtTy::DebugSet, FmtTy::DebugMap];

    /// The struct name in `core::fmt` (the def hash input and the
    /// printed name).
    pub fn name(self) -> &'static str {
        match self {
            FmtTy::Formatter => "Formatter",
            FmtTy::Arguments => "Arguments",
            FmtTy::DebugStruct => "DebugStruct",
            FmtTy::DebugTuple => "DebugTuple",
            FmtTy::DebugList => "DebugList",
            FmtTy::DebugSet => "DebugSet",
            FmtTy::DebugMap => "DebugMap",
        }
    }

    /// The kind a `fmt` path's last segment names, for type paths.
    pub fn by_name(name: &str) -> Option<FmtTy> {
        FmtTy::ALL.iter().copied().find(|k| k.name() == name)
    }

    /// True for the Debug builders, which hold `&mut Formatter` (one
    /// region slot in the borrow checker; `Arguments` holds one for
    /// its borrowed format arguments).
    pub fn is_builder(self) -> bool {
        !matches!(self, FmtTy::Formatter | FmtTy::Arguments)
    }
}

/// The def of one fmt struct (see `BuiltinDefs::fmt_tys`).
pub(super) fn fmt_def(k: FmtTy) -> DefId {
    builtin_def(&["core", "fmt"], k.name())
}

/// The fmt kind of a type, when it is one of these structs.
pub fn fmt_kind(tys: &impl TyIntern, t: Ty) -> Option<FmtTy> {
    match tys.builtin_of(t) {
        Some(BuiltinTy::Fmt(k)) => Some(k),
        _ => None,
    }
}

/// The type of one fmt struct.
pub fn fmt_ty(tys: &mut impl TyIntern, k: FmtTy) -> Ty {
    let def = tys.builtins().fmt_tys[k as usize];
    tys.mk_adt(def, &[])
}
