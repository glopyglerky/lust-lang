// Std iterator types named by path in a written type
// (`std::slice::Iter<'a, T>`, `std::vec::IntoIter<T>`,
// `std::iter::Map<I, fn(A) -> B>`, `std::str::Chars<'a>`). They are the
// builtin iterator Adts of `ty::iter`, so a value from `.iter()` and a
// written type agree. The type args lust adds after std's own (`Map`'s
// item `B`, `Copied`'s `T`) come from the written args: a `Map` or
// `FilterMap` function type must be a fn pointer, and an adapter's inner
// type must be a builtin iterator; other forms reject with L0641.
// In a region signature (`RegionMode::Keep`) the borrowing iterators
// carry their lifetime as the first arg, as user Adts do (the borrow
// checker reads one region slot for them, see
// `borrowck::regions::adt_regions`).
use crate::ast::{self, Ast, GenericArgs, PathSeg};
use crate::check::generics::{ResolveCx, Scope};
use crate::diag::{Diag, Span, not_yet};
use crate::intern::{Interner, Known};
use crate::ty::iter::{IterKind, SplitMode, item_ty, mk_iter, mk_split};
use crate::ty::*;

/// A std iterator type a written path names. `Chars` and `Bytes` have no
/// `IterKind` of their own (a `Bytes` value is a `Copied<slice::Iter<u8>>`),
/// and the `str` split family shares one kind told apart by mode.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StdIterPath {
    Chars,
    Bytes,
    Kind(IterKind),
    /// A `str` split iterator and whether its std name takes a pattern type.
    Split(SplitMode, bool),
}

/// The std iterator a full written path names, by its public std path
/// (`std::iter::Map`, `core::slice::Iter`, `alloc::vec::IntoIter`,
/// `std::collections::hash_map::Iter`). Private or bogus module paths do
/// not match, so they stay unknown types as in rustc.
pub fn iter_by_path(path: &[&str]) -> Option<StdIterPath> {
    use IterKind as K;
    use SplitMode as M;
    use StdIterPath as S;
    Some(match path {
        ["std" | "core", "str", "Chars"] => S::Chars,
        ["std" | "core", "str", "Bytes"] => S::Bytes,
        ["std" | "core", "str", "CharIndices"] => S::Kind(K::CharIndices),
        ["std" | "core", "str", "Split"] => S::Split(M::Split, true),
        ["std" | "core", "str", "RSplit"] => S::Split(M::RSplit, true),
        ["std" | "core", "str", "SplitTerminator"] => S::Split(M::SplitTerminator, true),
        ["std" | "core", "str", "RSplitTerminator"] => S::Split(M::RSplitTerminator, true),
        ["std" | "core", "str", "SplitN"] => S::Split(M::SplitN, true),
        ["std" | "core", "str", "RSplitN"] => S::Split(M::RSplitN, true),
        ["std" | "core", "str", "SplitInclusive"] => S::Split(M::SplitInclusive, true),
        ["std" | "core", "str", "SplitWhitespace"] => S::Split(M::SplitWhitespace, false),
        ["std" | "core", "str", "SplitAsciiWhitespace"] => S::Split(M::SplitAsciiWhitespace, false),
        ["std" | "core", "str", "Lines"] => S::Split(M::Lines, false),
        ["std" | "core", "slice", "Iter"] => S::Kind(K::SliceIter),
        ["std" | "core", "slice", "IterMut"] => S::Kind(K::SliceIterMut),
        ["std" | "core", "slice", "Windows"] => S::Kind(K::Windows),
        ["std" | "core", "slice", "Chunks"] => S::Kind(K::Chunks),
        ["std" | "core", "slice", "ChunksMut"] => S::Kind(K::ChunksMut),
        ["std" | "core", "slice", "ChunksExact"] => S::Kind(K::ChunksExact),
        ["std" | "core", "slice", "ChunksExactMut"] => S::Kind(K::ChunksExactMut),
        ["std" | "core", "str", "Utf8Chunks"] => S::Kind(K::Utf8Chunks),
        ["std" | "core", "str", "Utf8Chunk"] => S::Kind(K::Utf8Chunk),
        ["std" | "alloc", "vec", "IntoIter"] => S::Kind(K::VecIntoIter),
        ["std" | "alloc", "vec", "Drain"] => S::Kind(K::VecDrain),
        ["std" | "alloc", "string", "Drain"] => S::Kind(K::StringDrain),
        ["std" | "core", "ops", "Range"] => S::Kind(K::Range),
        ["std" | "core", "ops", "RangeInclusive"] => S::Kind(K::RangeInclusive),
        ["std" | "core", "ops", "RangeFrom"] => S::Kind(K::RangeFrom),
        ["std" | "core", "ops", "RangeTo"] => S::Kind(K::RangeTo),
        ["std" | "core", "ops", "RangeFull"] => S::Kind(K::RangeFull),
        ["std" | "core", "ops", "RangeToInclusive"] => S::Kind(K::RangeToInclusive),
        ["std" | "core", "iter", "Map"] => S::Kind(K::Map),
        ["std" | "core", "iter", "Filter"] => S::Kind(K::Filter),
        ["std" | "core", "iter", "Enumerate"] => S::Kind(K::Enumerate),
        ["std" | "core", "iter", "Rev"] => S::Kind(K::Rev),
        ["std" | "core", "iter", "Take"] => S::Kind(K::Take),
        ["std" | "core", "iter", "Skip"] => S::Kind(K::Skip),
        ["std" | "core", "iter", "Chain"] => S::Kind(K::Chain),
        ["std" | "core", "iter", "Zip"] => S::Kind(K::Zip),
        ["std" | "core", "iter", "Copied"] => S::Kind(K::Copied),
        ["std" | "core", "iter", "Cloned"] => S::Kind(K::Cloned),
        ["std" | "core", "iter", "FilterMap"] => S::Kind(K::FilterMap),
        ["std" | "core", "iter", "MapWhile"] => S::Kind(K::MapWhile),
        ["std" | "core", "iter", "TakeWhile"] => S::Kind(K::TakeWhile),
        ["std" | "core", "iter", "SkipWhile"] => S::Kind(K::SkipWhile),
        ["std" | "core", "iter", "StepBy"] => S::Kind(K::StepBy),
        ["std" | "core", "iter", "Peekable"] => S::Kind(K::Peekable),
        ["std" | "core", "iter", "Inspect"] => S::Kind(K::Inspect),
        ["std" | "core", "iter", "Flatten"] => S::Kind(K::Flatten),
        ["std" | "core", "iter", "RepeatN"] => S::Kind(K::RepeatN),
        ["std", "env", "Args"] => S::Kind(K::Args),
        ["std" | "core", "option", "Iter"] => S::Kind(K::OptionIter),
        ["std" | "core", "option", "IterMut"] => S::Kind(K::OptionIterMut),
        ["std" | "core", "option", "IntoIter"] => S::Kind(K::OptionIntoIter),
        ["std" | "core", "result", "Iter"] => S::Kind(K::ResultIter),
        ["std" | "core", "result", "IterMut"] => S::Kind(K::ResultIterMut),
        ["std" | "core", "result", "IntoIter"] => S::Kind(K::ResultIntoIter),
        ["std" | "core", "array", "IntoIter"] => S::Kind(K::ArrayIntoIter),
        ["std", "sync", "mpsc", "Iter"] => S::Kind(K::RecvIter),
        ["std", "sync", "mpsc", "TryIter"] => S::Kind(K::RecvTryIter),
        ["std", "sync", "mpsc", "IntoIter"] => S::Kind(K::RecvIntoIter),
        ["std", "collections", "hash_map", "Iter"] => S::Kind(K::MapIter),
        ["std", "collections", "hash_map", "IterMut"] => S::Kind(K::MapIterMut),
        ["std", "collections", "hash_map", "Keys"] => S::Kind(K::MapKeys),
        ["std", "collections", "hash_map", "Values"] => S::Kind(K::MapValues),
        ["std", "collections", "hash_map", "ValuesMut"] => S::Kind(K::MapValuesMut),
        ["std", "collections", "hash_map", "IntoIter"] => S::Kind(K::MapIntoIter),
        ["std", "collections", "hash_map", "IntoKeys"] => S::Kind(K::MapIntoKeys),
        ["std", "collections", "hash_map", "IntoValues"] => S::Kind(K::MapIntoValues),
        ["std", "collections", "hash_set", "Iter"] => S::Kind(K::SetIter),
        ["std", "collections", "hash_set", "IntoIter"] => S::Kind(K::SetIntoIter),
        ["std", "collections", "hash_map", "Drain"] => S::Kind(K::MapDrain),
        ["std", "collections", "hash_set", "Drain"] => S::Kind(K::SetDrain),
        _ => return None,
    })
}

/// True for the kinds whose std name takes one lifetime argument (the
/// borrowing iterators of `borrowck::regions::adt_regions`).
fn has_lifetime(k: IterKind) -> bool {
    use IterKind as K;
    matches!(
        k,
        K::SliceIter
            | K::SliceIterMut
            | K::RecvIter
            | K::RecvTryIter
            | K::Windows
            | K::Chunks
            | K::ChunksMut
            | K::ChunksExact
            | K::ChunksExactMut
            | K::Utf8Chunks
            | K::Utf8Chunk
            | K::CharIndices
            | K::VecDrain
            | K::StringDrain
            | K::OptionIter
            | K::OptionIterMut
            | K::ResultIter
            | K::ResultIterMut
            | K::StrSplit
            | K::SliceSplit
            | K::ChunkBy
            | K::ChunkByMut
            | K::SliceSplitMut
            | K::MapIter
            | K::MapIterMut
            | K::MapKeys
            | K::MapValues
            | K::MapValuesMut
            | K::SetIter
            | K::MapDrain
            | K::SetDrain
    )
}

/// True for a written split pattern type lust runs: `char`, a char array,
/// a `&str` kind of needle, a char slice, or a `fn(char) -> bool`.
fn pattern_ty_ok(tys: &mut impl TyIntern, t: Ty) -> bool {
    match tys.kind(t) {
        TyKind::Char => true,
        TyKind::Array(e, _) => e == CHAR,
        TyKind::Ref(_, _, inner) => match tys.kind(inner) {
            TyKind::Str => true,
            TyKind::Ref(_, _, x) => tys.kind(x) == TyKind::Str,
            TyKind::Array(e, _) | TyKind::Slice(e) => e == CHAR,
            _ => tys.is_string_ty(inner),
        },
        TyKind::FnPtr(sig) => {
            let s = tys.sig(sig).value;
            let ins = tys.tys_of(s.inputs);
            ins.len() == 1 && ins[0] == CHAR && s.output == BOOL
        }
        _ => false,
    }
}

/// The fn pointer a function adapter's second written arg must be, checked
/// against the inner item; None when it is not a matching fn pointer.
fn pred_fn_ok(tys: &mut impl TyIntern, f: Ty, inner: Ty, out: Ty) -> bool {
    let TyKind::FnPtr(sig) = tys.kind(f) else { return false };
    let s = tys.sig(sig).value;
    let ins: Vec<Ty> = tys.tys_of(s.inputs).to_vec();
    let Some(item) = item_ty(tys, inner) else { return false };
    let want = tys.mk_ref(false, item);
    ins.len() == 1 && ins[0] == want && s.output == out
}

/// True when core implements `DoubleEndedIterator` for builtin iterator
/// `t` (the element bounds aside: the adapters' `Iterator` impls prove
/// those). Conservative: the str and slice splits and `Flatten` answer
/// false. Lowering has no `next_back` for some kinds this accepts (`Skip`):
/// `lust build` rejects a `Rev` over one when the code iterates it.
fn written_double_ended(tys: &impl TyIntern, t: Ty) -> bool {
    use IterKind as K;
    if tys.is_chars_ty(t) {
        return true;
    }
    let Some((k, a)) = crate::ty::iter::iter_parts(tys, t) else { return false };
    match k {
        K::SliceIter
        | K::SliceIterMut
        | K::VecIntoIter
        | K::Range
        | K::RangeInclusive
        | K::Windows
        | K::Chunks
        | K::ChunksMut
        | K::ChunksExact
        | K::ChunksExactMut
        | K::CharIndices
        | K::VecDrain
        | K::StringDrain
        | K::ArrayIntoIter
        | K::RepeatN
        | K::OptionIntoIter
        | K::OptionIter
        | K::OptionIterMut
        | K::ResultIter
        | K::ResultIterMut
        | K::ResultIntoIter
        | K::Args => true,
        K::Map | K::Filter | K::FilterMap | K::Rev | K::Copied | K::Cloned | K::Inspect | K::Peekable => written_double_ended(tys, a[0]),
        K::Chain => written_double_ended(tys, a[0]) && written_double_ended(tys, a[1]),
        // `I: DoubleEndedIterator + ExactSizeIterator`.
        K::Enumerate | K::Take | K::Skip | K::StepBy => written_double_ended(tys, a[0]) && written_exact_size(tys, a[0]),
        K::Zip => (0..2).all(|i| written_double_ended(tys, a[i]) && written_exact_size(tys, a[i])),
        _ => false,
    }
}

/// True when core implements `ExactSizeIterator` for builtin iterator `t`
/// (the element bounds aside). Conservative like `written_double_ended`.
fn written_exact_size(tys: &impl TyIntern, t: Ty) -> bool {
    use IterKind as K;
    let Some((k, a)) = crate::ty::iter::iter_parts(tys, t) else { return false };
    match k {
        K::Range | K::RangeInclusive => match tys.kind(a[0]) {
            TyKind::Int(i) => crate::check::iter::exact_range_int(i, k == K::RangeInclusive),
            _ => false,
        },
        K::SliceIter
        | K::SliceIterMut
        | K::VecIntoIter
        | K::VecDrain
        | K::ArrayIntoIter
        | K::RepeatN
        | K::Windows
        | K::Chunks
        | K::ChunksMut
        | K::ChunksExact
        | K::ChunksExactMut
        | K::OptionIter
        | K::OptionIterMut
        | K::OptionIntoIter
        | K::ResultIter
        | K::ResultIterMut
        | K::ResultIntoIter
        | K::Args => true,
        K::Map | K::Enumerate | K::Rev | K::Take | K::Skip | K::StepBy | K::Copied | K::Cloned | K::Peekable | K::Inspect => {
            written_exact_size(tys, a[0])
        }
        K::Zip => written_exact_size(tys, a[0]) && written_exact_size(tys, a[1]),
        _ => false,
    }
}

/// Resolves a std iterator type path (`iter_by_path` matched it) with the
/// written args of its last segment.
#[allow(clippy::too_many_arguments)]
pub fn resolve_iter_path(
    ast: &Ast,
    interner: &Interner,
    known: &Known,
    cx: &ResolveCx,
    tys: &mut impl TyIntern,
    scope: &Scope,
    target: StdIterPath,
    seg: &PathSeg,
    span: Span,
) -> Result<Ty, Diag> {
    let shown = crate::intern::source_name(interner.get(seg.name)).to_string();
    let (mut ty_args, mut lts): (Vec<Ty>, Vec<ast::Lifetime>) = (Vec::new(), Vec::new());
    // `array::IntoIter<T, N>` (fx-array-iter) is the one std iterator
    // written with a const argument.
    let mut const_args: Vec<crate::ty::ConstId> = Vec::new();
    match &seg.args {
        GenericArgs::None => {}
        GenericArgs::Angle(args) => {
            for a in args {
                match a {
                    ast::GenericArg::Type(t) => {
                        ty_args.push(crate::check::generics::resolve_scoped(ast, interner, known, cx, tys, *t, scope)?)
                    }
                    ast::GenericArg::Lifetime(lt) => lts.push(*lt),
                    ast::GenericArg::Const(e) if target == StdIterPath::Kind(IterKind::ArrayIntoIter) => {
                        const_args.push(crate::check::generics::resolve_const_arg(ast, interner, known, tys, scope, *e, span)?)
                    }
                    _ => {
                        return Err(Diag::new(
                            "L0110",
                            span,
                            format!("`{shown}` takes only type and lifetime arguments"),
                            "Remove the other arguments.",
                        ));
                    }
                }
            }
        }
        GenericArgs::Parenthesized { .. } => {
            return Err(Diag::new("L0110", span, "parenthesized arguments are only for Fn traits", "Use angle arguments."));
        }
    }
    let (want, n_lts) = match target {
        StdIterPath::Chars | StdIterPath::Bytes => (0, 1),
        StdIterPath::Split(_, has_pat) => (usize::from(has_pat), 1),
        StdIterPath::Kind(k) => (k.shown_args(), usize::from(has_lifetime(k))),
    };
    if lts.len() > n_lts {
        return Err(Diag::new(
            "L0110",
            span,
            format!("`{shown}` takes {n_lts} lifetime argument(s), but {} were given", lts.len()),
            "Match the number of lifetime parameters in the definition.",
        ));
    }
    if ty_args.len() != want {
        return Err(Diag::new(
            "L0110",
            span,
            format!("`{shown}` takes {want} generic argument(s), but {} were given", ty_args.len()),
            "Match the number of parameters in the definition.",
        ));
    }
    if target == StdIterPath::Kind(IterKind::ArrayIntoIter) && const_args.len() != 1 {
        return Err(Diag::new(
            "L0110",
            span,
            format!("`{shown}` takes 1 const argument(s), but {} were given", const_args.len()),
            "Match the number of parameters in the definition.",
        ));
    }
    // In a region signature the written args keep their regions; the
    // checks and the derived item args read their erased forms.
    let keep = matches!(scope.regions, crate::check::generics::RegionMode::Keep { .. });
    let mut erased_args: Vec<Ty> = Vec::with_capacity(ty_args.len());
    if keep {
        let erased_scope = scope.erased();
        if let GenericArgs::Angle(args) = &seg.args {
            for a in args {
                if let ast::GenericArg::Type(t) = a {
                    erased_args.push(crate::check::generics::resolve_scoped(ast, interner, known, cx, tys, *t, &erased_scope)?);
                }
            }
        }
    } else {
        erased_args.clone_from(&ty_args);
    }
    let unsupported = |what: &str| not_yet("L0641", span, &format!("`{shown}` {what}"));
    // `Bytes` is a `Copied<slice::Iter<u8>>` value: its one lifetime is the
    // inner slice iterator's, so the region form nests instead of leading.
    if target == StdIterPath::Bytes {
        let it = mk_iter(tys, IterKind::SliceIter, &[int(IntTy::U8)]);
        let erased = mk_iter(tys, IterKind::Copied, &[it, int(IntTy::U8)]);
        if !keep {
            return Ok(erased);
        }
        let (copied_def, _) = tys.adt_parts(erased).expect("a builtin iterator is an Adt");
        let (slice_def, _) = tys.adt_parts(it).expect("a builtin iterator is an Adt");
        let lt = crate::check::generics::scope_region(interner, scope, lts.first().copied(), span)?;
        let inner = tys.mk_adt(slice_def, &[GenericArg::Lifetime(lt), GenericArg::Ty(int(IntTy::U8))]);
        return Ok(tys.mk_adt(copied_def, &[GenericArg::Ty(inner), GenericArg::Ty(int(IntTy::U8))]));
    }
    let borrows = n_lts == 1;
    let (erased, extra) = match target {
        StdIterPath::Chars => (CHARS, Vec::new()),
        StdIterPath::Bytes => unreachable!("handled above"),
        StdIterPath::Split(mode, has_pat) => {
            let pat = if has_pat {
                if !pattern_ty_ok(tys, erased_args[0]) {
                    return Err(unsupported("with a pattern type lust does not take"));
                }
                erased_args[0]
            } else {
                CHAR
            };
            let erased = mk_split(tys, pat, mode);
            let (_, parts) = crate::ty::iter::iter_parts(tys, erased).expect("a split type is a builtin iterator");
            // The mode marker follows the written args; the pattern-less
            // modes also store their `char` placeholder there.
            let extra = if has_pat { vec![parts[1]] } else { vec![parts[0], parts[1]] };
            (erased, extra)
        }
        StdIterPath::Kind(k) => {
            use IterKind as K;
            // Adapters wrap a builtin iterator (their item and fields need it).
            let adapter = k.is_adapter();
            if adapter && !crate::ty::iter::is_iter(tys, erased_args[0]) {
                return Err(unsupported("over a type that is not a std iterator lust knows"));
            }
            if matches!(k, K::Chain | K::Zip) && !crate::ty::iter::is_iter(tys, erased_args[1]) {
                return Err(unsupported("over a type that is not a std iterator lust knows"));
            }
            // `impl<I: DoubleEndedIterator> Iterator for Rev<I>`: a `Rev` that
            // is no iterator would let its uses skip that bound.
            if k == K::Rev && !written_double_ended(tys, erased_args[0]) {
                return Err(unsupported("over an iterator lust does not know to be double-ended"));
            }
            if k.is_range()
                && k != K::RangeFull
                && !erased_args.first().is_some_and(|&t| matches!(tys.kind(t), TyKind::Int(_) | TyKind::Char))
            {
                return Err(unsupported("over a type that is not an integer or `char`"));
            }
            let mut extra: Vec<Ty> = Vec::new();
            match k {
                // The second lust arg is the array type itself.
                K::ArrayIntoIter => {
                    let arr = tys.intern(TyKind::Array(erased_args[0], const_args[0]));
                    extra.push(arr);
                }
                K::Map | K::FilterMap | K::MapWhile => {
                    let TyKind::FnPtr(sig) = tys.kind(erased_args[1]) else {
                        return Err(unsupported("with a function type that is not a fn pointer"));
                    };
                    let s = tys.sig(sig).value;
                    let item = item_ty(tys, erased_args[0]);
                    if tys.tys_of(s.inputs).len() != 1 || item.is_none_or(|it| tys.tys_of(s.inputs)[0] != it) {
                        return Err(unsupported("with a fn pointer that does not take the inner item"));
                    }
                    let out = if k == K::Map {
                        s.output
                    } else {
                        match tys.option_elem(s.output) {
                            Some(b) => b,
                            None => return Err(unsupported("with a fn pointer that does not return an `Option`")),
                        }
                    };
                    extra.push(out);
                }
                K::Filter | K::TakeWhile | K::SkipWhile => {
                    if !pred_fn_ok(tys, erased_args[1], erased_args[0], BOOL) {
                        return Err(unsupported("with a predicate that is not a matching fn pointer"));
                    }
                }
                K::Inspect => {
                    if !pred_fn_ok(tys, erased_args[1], erased_args[0], UNIT) {
                        return Err(unsupported("with a function that is not a matching fn pointer"));
                    }
                }
                K::Copied | K::Cloned => {
                    let inner = item_ty(tys, erased_args[0]).and_then(|it| match tys.kind(it) {
                        TyKind::Ref(_, Mut::Shared, t) => Some(t),
                        _ => None,
                    });
                    match inner {
                        Some(t) => extra.push(t),
                        None => return Err(unsupported("over an iterator whose items are not shared references")),
                    }
                }
                // The one lust arg is the fixed `u8` element marker.
                K::Utf8Chunks | K::Utf8Chunk => extra.push(int(IntTy::U8)),
                // The second lust arg is the inner items' iterator.
                K::Flatten => {
                    let u = item_ty(tys, erased_args[0]).and_then(|it| crate::ty::iter::as_iter_ty(tys, it));
                    match u {
                        Some(u) => extra.push(u),
                        None => return Err(unsupported("over an iterator whose items lust cannot flatten")),
                    }
                }
                _ => {}
            }
            let mut all = erased_args.clone();
            all.extend(extra.iter().copied());
            (mk_iter(tys, k, &all), extra)
        }
    };
    if !keep {
        return Ok(erased);
    }
    // The region signature form: the lifetime first (borrowing iterators
    // only), then the written args with their regions, then the derived ones.
    let (def, _) = tys.adt_parts(erased).expect("a builtin iterator is an Adt");
    let mut all: Vec<GenericArg> = Vec::new();
    if borrows {
        all.push(GenericArg::Lifetime(crate::check::generics::scope_region(interner, scope, lts.first().copied(), span)?));
    }
    all.extend(ty_args.iter().map(|&t| GenericArg::Ty(t)));
    all.extend(extra.iter().map(|&t| GenericArg::Ty(t)));
    Ok(tys.mk_adt(def, &all))
}
