// Builtin iterator types (W-STD-A phase 3; docs/ARCH.md 3.4 and 4.3).
//
// ARCH puts the std iterator structs in `library/core` source. That
// needs lifetime params on structs (`slice::Iter<'a, T>`) and closures
// stored in Adts, which the checker does not have yet, so until then
// each std iterator struct is a builtin Adt with its own def. A value
// is laid out as a tuple of the fields real std declares, in
// declaration order, so drop order matches Rust. Check types the
// methods (`check/iter.rs`), borrowck sees each method as one opaque
// builtin call, and codegen first expands the calls into plain IR
// with Rust 1.99's `next` semantics (`lower/iter.rs`).
//
// Type args are the std struct's own args, plus the item type last
// for the adapters whose item a closure returns (`Map`, `FilterMap`):
// item types then never need a closure signature lookup. `str::Chars`
// predates this family and stays its own builtin (`BuiltinTy::Chars`),
// but it is an iterator source here too.
use super::*;

/// One std iterator struct lust knows by def.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IterKind {
    /// `core::slice::Iter<T>`: args `[T]`, item `&T`.
    SliceIter,
    /// `core::slice::IterMut<T>`: args `[T]`, item `&mut T`.
    SliceIterMut,
    /// `alloc::vec::IntoIter<T>`: args `[T]`, item `T`.
    VecIntoIter,
    /// `core::ops::Range<T>` over an integer: args `[T]`, item `T`.
    Range,
    /// `core::ops::RangeInclusive<T>` over an integer: args `[T]`.
    RangeInclusive,
    /// `Map<I, F>`: args `[I, F, B]` with `B` the closure output.
    Map,
    /// `Filter<I, P>`: args `[I, P]`.
    Filter,
    /// `Enumerate<I>`: args `[I]`, item `(usize, I::Item)`.
    Enumerate,
    /// `Rev<I>`: args `[I]`.
    Rev,
    /// `Take<I>`: args `[I]`.
    Take,
    /// `Skip<I>`: args `[I]`.
    Skip,
    /// `Chain<A, B>`: args `[A, B]`.
    Chain,
    /// `Zip<A, B>`: args `[A, B]`, item `(A::Item, B::Item)`.
    Zip,
    /// `Copied<I>`: args `[I, T]` with `T` the item (`I::Item = &T`).
    Copied,
    /// `Cloned<I>`: args `[I, T]` with `T` the item (`I::Item = &T`).
    Cloned,
    /// `FilterMap<I, F>`: args `[I, F, B]` with `F` returning `Option<B>`.
    FilterMap,
    /// `TakeWhile<I, P>`: args `[I, P]`.
    TakeWhile,
    /// `SkipWhile<I, P>`: args `[I, P]`.
    SkipWhile,
    /// `StepBy<I>`: args `[I]`.
    StepBy,
    /// `Peekable<I>`: args `[I]`.
    Peekable,
    /// `Inspect<I, F>`: args `[I, F]`.
    Inspect,
    /// `std::sync::mpsc::Iter<'a, T>` (`rx.iter()`, `for x in &rx`):
    /// args `[T]`, item `T`; blocks in `recv` (W-THREADS).
    RecvIter,
    /// `std::sync::mpsc::TryIter<'a, T>` (`rx.try_iter()`): args `[T]`.
    RecvTryIter,
    /// `std::sync::mpsc::IntoIter<T>` (`for x in rx`): args `[T]`.
    RecvIntoIter,
    // The hashed-collection iterators (W-STD-B; cgen/hash.rs). Map
    // kinds take args `[K, V]`, set kinds `[T]`. Their state is the
    // hash core's raw iterator (`fields` is empty: `next` and the
    // length are builtin calls).
    /// `hash_map::Iter<K, V>`: item `(&K, &V)`.
    MapIter,
    /// `hash_map::IterMut<K, V>`: item `(&K, &mut V)`.
    MapIterMut,
    /// `hash_map::Keys<K, V>`: item `&K`.
    MapKeys,
    /// `hash_map::Values<K, V>`: item `&V`.
    MapValues,
    /// `hash_map::ValuesMut<K, V>`: item `&mut V`.
    MapValuesMut,
    /// `hash_map::IntoIter<K, V>`: item `(K, V)`.
    MapIntoIter,
    /// `hash_map::IntoKeys<K, V>`: item `K`.
    MapIntoKeys,
    /// `hash_map::IntoValues<K, V>`: item `V`.
    MapIntoValues,
    /// `hash_set::Iter<T>`: item `&T`.
    SetIter,
    /// `hash_set::IntoIter<T>`: item `T`.
    SetIntoIter,
    /// `array::IntoIter<T, N>` (fx-array-iter): args `[T, A]` with
    /// `A = [T; N]`, item `T`, for every `T`. Its C type
    /// (`CTy::ArrayIter`) drops only the stored range the indexes
    /// still span.
    ArrayIntoIter,
    /// `iter::RepeatN<T>` (W-STD-B): args `[T]`, item `T`. As core's
    /// `Option<RepeatNInner>`: the element while the count is above 0,
    /// cloned for each item but the last, which moves it out.
    RepeatN,
    /// Any other iterator, reached through its own `Iterator::next`
    /// (W-STD-B, check/iter.rs `by_next_item`): args `[I, Item]`, the
    /// iterator value itself as the one field. A user type with an
    /// `Iterator` impl, or in a template a type parameter or projection
    /// with an `Iterator` bound; adapters and consumers wrap it once.
    /// Never named in source: messages print `I`.
    ByNext,
    /// `core::slice::Windows<T>` (W-STD-B): args `[T]`, item `&[T]`;
    /// (the slice left, the window size).
    Windows,
    /// `core::str::CharIndices` (W-STD-B): no args, item `(usize,
    /// char)`; (the `Chars` left, the byte offset of its front), as std
    /// stores it.
    CharIndices,
    /// `Flatten<I>` (W-STD-B): args `[I, U]` with `U` the inner items'
    /// `IntoIter`; (iter, frontiter, backiter), as std's FlattenCompat,
    /// whose `Fuse<I>` drops the outer iterator once it ends (`None`
    /// here). `flat_map(f)` is `Flatten<Map<I, F>>`, which is what std's
    /// `FlatMap` runs.
    Flatten,
    /// `option::IntoIter<T>` (W-STD-B): args `[T]`, the option itself.
    OptionIntoIter,
    /// `option::Iter<T>` (fx-option-rest): args `[T]`, item `&T`;
    /// the field is `Option<&T>`, as core stores it.
    OptionIter,
    /// `option::IterMut<T>`: args `[T]`, item `&mut T`.
    OptionIterMut,
    /// `result::Iter<T>`: args `[T]`, item `&T` (the `Ok` value).
    ResultIter,
    /// `result::IterMut<T>`: args `[T]`, item `&mut T`.
    ResultIterMut,
    /// `result::IntoIter<T>`: args `[T]`, item `T`; the field is the
    /// `Ok` value as an `Option<T>` (an `Err` drops at conversion).
    ResultIntoIter,
    /// The `str` split iterators (W-STD-B): `Split`, `RSplit`,
    /// `SplitTerminator`, `RSplitTerminator`, `SplitN`, `RSplitN`,
    /// `SplitInclusive`, `SplitWhitespace`, `SplitAsciiWhitespace`, and
    /// `Lines`, by `SplitMode`. Args `[P, M]`: the pattern's type and the
    /// mode as `[u8; mode]` (a marker, never a value). Item `&str`.
    /// Fields: the runtime's `lust_split` state (haystack, start, end,
    /// front and back search positions, flags, count left), then the
    /// pattern (runtime/core.c `lust_split_step`).
    StrSplit,
    /// The `str` matches iterators (tf-str): `Matches`, `RMatches`,
    /// `MatchIndices`, and `RMatchIndices`, by `MatchesMode`. Args
    /// `[P, M]` as `StrSplit`'s. Fields: the same `lust_split` state
    /// (only haystack, search positions, and flags are used), then
    /// the pattern (runtime/core.c `lust_matches_step`).
    StrMatches,
    /// `core::str::EncodeUtf16` (tf-str): no args, item `u16`;
    /// (the `Chars` left, a buffered low surrogate or 0).
    EncodeUtf16,
    /// `alloc::vec::Drain<T>`: args `[T]`, item `T`. Holds the
    /// drained Vec mutably; its drop removes what it did not yield.
    VecDrain,
    /// `alloc::string::Drain`: no args, item `char`. Holds the
    /// drained String mutably.
    StringDrain,
    /// `core::ops::RangeFrom<T>`: args `[T]`. Not an iterator here
    /// (std's Step iterator is not modeled); a `RangeBounds` value.
    RangeFrom,
    /// `core::ops::RangeTo<T>`: args `[T]`. A `RangeBounds` value.
    RangeTo,
    /// `core::ops::RangeFull`: no args. A `RangeBounds` value.
    RangeFull,
    /// `core::ops::RangeToInclusive<T>`: args `[T]`. A `RangeBounds` value.
    RangeToInclusive,
    /// `core::slice::Chunks<T>` (tf-slice): args `[T]`, item `&[T]`;
    /// (the slice left, the chunk size), as std stores it.
    Chunks,
    /// `core::slice::ChunksMut<T>`: item `&mut [T]`.
    ChunksMut,
    /// `core::slice::ChunksExact<T>`: (full chunks left, remainder,
    /// chunk size), as std stores it.
    ChunksExact,
    /// `core::slice::ChunksExactMut<T>`: item `&mut [T]`.
    ChunksExactMut,
    /// `core::str::Utf8Chunks`: args `[u8]` (never shown), item
    /// `Utf8Chunk`; (the bytes left).
    Utf8Chunks,
    /// `core::str::Utf8Chunk`: args `[u8]` (never shown); (valid
    /// `&str`, invalid `&[u8]`). Not an iterator: the item type of
    /// `Utf8Chunks`, carried here like the range kinds.
    Utf8Chunk,
    /// The short-circuit wrapper of a `collect` into `Result<C, E>`
    /// or `Option<C>` (F3; core's `ProcessResults` in std). Args
    /// `[I, E, T]`: the source iterator, the error type (`()` marks
    /// the `Option` form), and the item `T`. Fields: the source,
    /// then the first failure seen (`Option<E>`), in std's
    /// declaration order for drop. Never named in source: only the
    /// `collect` expansion builds one (lower/iter.rs).
    TryCollect,
    /// `std::env::Args`: no args, item `String`; (front, back)
    /// indexes into the process arguments. `next` and `next_back`
    /// check the UTF-8 of the argument they take and panic as std's
    /// `into_string().unwrap()` does (runtime `lust_env_arg`).
    Args,
    /// The slice splits over a predicate (`split`, `rsplit`, `splitn`,
    /// `rsplitn`, `split_inclusive`), by `SliceSplitMode`. Args
    /// `[T, P, M]`: the element, the predicate's type, and the mode as
    /// `[u8; mode]` (a marker, never a value). Item `&[T]`. Fields
    /// (v, pred, finished, count), as core's `Split` and
    /// `GenericSplitN` store them (`count` serves the N forms only).
    SliceSplit,
    /// The `_mut` forms of `SliceSplit`: item `&mut [T]`.
    SliceSplitMut,
    /// `MapWhile<I, P>` (R22): args `[I, P, B]` with `P` returning
    /// `Option<B>`; fields (iter, predicate), as core's. Not fused.
    MapWhile,
    /// `hash_map::Drain<'a, K, V>` (R20): args `[K, V]`, item `(K, V)`.
    /// A hash iterator (cgen/hash.rs): it holds the map's table, moved
    /// out as hashbrown's `RawDrain` does, and puts it back emptied
    /// when it drops.
    MapDrain,
    /// `hash_set::Drain<'a, T>`: args `[T]`, item `T`.
    SetDrain,
    /// `slice::ChunkBy<'a, T, P>` (R13): args `[T, P]`, item `&[T]`;
    /// fields (slice, predicate), as core's.
    ChunkBy,
    /// `slice::ChunkByMut<'a, T, P>`: item `&mut [T]`.
    ChunkByMut,
}

/// Which std struct a `StrSplit` is (its `[u8; mode]` type argument).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SplitMode {
    Split,
    RSplit,
    SplitTerminator,
    RSplitTerminator,
    SplitN,
    RSplitN,
    SplitInclusive,
    SplitWhitespace,
    SplitAsciiWhitespace,
    Lines,
}

impl SplitMode {
    const ALL: [SplitMode; 10] = [
        SplitMode::Split,
        SplitMode::RSplit,
        SplitMode::SplitTerminator,
        SplitMode::RSplitTerminator,
        SplitMode::SplitN,
        SplitMode::RSplitN,
        SplitMode::SplitInclusive,
        SplitMode::SplitWhitespace,
        SplitMode::SplitAsciiWhitespace,
        SplitMode::Lines,
    ];
}

/// A `StrSplit` type: pattern type `p` (CHAR for the whitespace and
/// line modes, which have none) and its mode.
pub fn mk_split(tys: &mut impl TyIntern, p: Ty, mode: SplitMode) -> Ty {
    let c = tys.intern_const(ConstKind::Int(mode as u128));
    let marker = tys.intern(TyKind::Array(int(IntTy::U8), c));
    mk_iter(tys, IterKind::StrSplit, &[p, marker])
}

/// The mode of a `StrSplit` type's marker argument.
pub fn split_mode(tys: &impl TyIntern, marker: Ty) -> Option<SplitMode> {
    let TyKind::Array(_, c) = tys.kind(marker) else { return None };
    let ConstKind::Int(n) = tys.const_kind(c) else { return None };
    SplitMode::ALL.get(n as usize).copied()
}

/// Which std struct a `StrMatches` is (its `[u8; mode]` marker).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatchesMode {
    Matches,
    RMatches,
    MatchIndices,
    RMatchIndices,
}

impl MatchesMode {
    const ALL: [MatchesMode; 4] = [MatchesMode::Matches, MatchesMode::RMatches, MatchesMode::MatchIndices, MatchesMode::RMatchIndices];

    /// True for the `(usize, &str)` item forms.
    pub fn indexed(self) -> bool {
        matches!(self, MatchesMode::MatchIndices | MatchesMode::RMatchIndices)
    }

    /// True for the forms whose `next` searches from the back.
    pub fn reversed(self) -> bool {
        matches!(self, MatchesMode::RMatches | MatchesMode::RMatchIndices)
    }
}

/// A `StrMatches` type for pattern type `p` in `mode`.
pub fn mk_matches(tys: &mut impl TyIntern, p: Ty, mode: MatchesMode) -> Ty {
    let c = tys.intern_const(ConstKind::Int(mode as u128));
    let marker = tys.intern(TyKind::Array(int(IntTy::U8), c));
    mk_iter(tys, IterKind::StrMatches, &[p, marker])
}

/// The mode of a `StrMatches` type's marker argument.
pub fn matches_mode(tys: &impl TyIntern, marker: Ty) -> Option<MatchesMode> {
    let TyKind::Array(_, c) = tys.kind(marker) else { return None };
    let ConstKind::Int(n) = tys.const_kind(c) else { return None };
    MatchesMode::ALL.get(n as usize).copied()
}

/// Which core struct a `SliceSplit` (or `SliceSplitMut`) is: its
/// `[u8; mode]` type argument.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SliceSplitMode {
    Split,
    RSplit,
    SplitN,
    RSplitN,
    SplitInclusive,
}

impl SliceSplitMode {
    const ALL: [SliceSplitMode; 5] =
        [SliceSplitMode::Split, SliceSplitMode::RSplit, SliceSplitMode::SplitN, SliceSplitMode::RSplitN, SliceSplitMode::SplitInclusive];

    /// The mode a slice method builds, with its mutability; None for
    /// other names.
    pub fn of_method(name: &str) -> Option<(SliceSplitMode, bool)> {
        use SliceSplitMode as M;
        Some(match name {
            "split" => (M::Split, false),
            "split_mut" => (M::Split, true),
            "rsplit" => (M::RSplit, false),
            "rsplit_mut" => (M::RSplit, true),
            "splitn" => (M::SplitN, false),
            "splitn_mut" => (M::SplitN, true),
            "rsplitn" => (M::RSplitN, false),
            "rsplitn_mut" => (M::RSplitN, true),
            "split_inclusive" => (M::SplitInclusive, false),
            "split_inclusive_mut" => (M::SplitInclusive, true),
            _ => return None,
        })
    }

    /// True for the forms with `next_back` (not the N forms).
    pub fn double_ended(self) -> bool {
        !matches!(self, SliceSplitMode::SplitN | SliceSplitMode::RSplitN)
    }
}

/// A `SliceSplit` (or, `mutable`, `SliceSplitMut`) type over element
/// `t` and predicate type `p` in `mode`.
pub fn mk_slice_split(tys: &mut impl TyIntern, t: Ty, p: Ty, mode: SliceSplitMode, mutable: bool) -> Ty {
    let c = tys.intern_const(ConstKind::Int(mode as u128));
    let marker = tys.intern(TyKind::Array(int(IntTy::U8), c));
    let k = if mutable { IterKind::SliceSplitMut } else { IterKind::SliceSplit };
    mk_iter(tys, k, &[t, p, marker])
}

/// The mode of a `SliceSplit` type's marker argument.
pub fn slice_split_mode(tys: &impl TyIntern, marker: Ty) -> Option<SliceSplitMode> {
    let TyKind::Array(_, c) = tys.kind(marker) else { return None };
    let ConstKind::Int(n) = tys.const_kind(c) else { return None };
    SliceSplitMode::ALL.get(n as usize).copied()
}

/// One stored field of a builtin iterator, over the type's args.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    /// The type arg at this index.
    Arg(u8),
    /// `usize`.
    Usize,
    /// `bool`.
    Bool,
    /// `Option<arg>`.
    OptArg(u8),
    /// `Option<&arg>`, or `Option<&mut arg>` when the flag is set
    /// (the `option` and `result` borrowing iterators).
    OptRef(u8, bool),
    /// `&[arg]`, or `&mut [arg]` when the flag is set.
    SliceRef(u8, bool),
    /// `Vec<arg>`.
    VecArg(u8),
    /// `Option<Option<item of arg>>`: the peeked slot of `Peekable`.
    PeekSlot(u8),
    /// `&Receiver<arg>`, or `Receiver<arg>` by value when the flag is
    /// set (the mpsc iterators, W-THREADS).
    Recv(u8, bool),
    /// `str::Chars`.
    Chars,
    /// `&str`.
    Str,
    /// `&mut Vec<arg>`: the drained Vec of `vec::Drain`.
    VecRefMut(u8),
    /// `&mut String`: the drained String of `string::Drain`.
    StringRefMut,
}

impl IterKind {
    pub const ALL: [IterKind; 70] = [
        IterKind::SliceIter,
        IterKind::SliceIterMut,
        IterKind::VecIntoIter,
        IterKind::Range,
        IterKind::RangeInclusive,
        IterKind::Map,
        IterKind::Filter,
        IterKind::Enumerate,
        IterKind::Rev,
        IterKind::Take,
        IterKind::Skip,
        IterKind::Chain,
        IterKind::Zip,
        IterKind::Copied,
        IterKind::Cloned,
        IterKind::FilterMap,
        IterKind::TakeWhile,
        IterKind::SkipWhile,
        IterKind::StepBy,
        IterKind::Peekable,
        IterKind::Inspect,
        IterKind::RecvIter,
        IterKind::RecvTryIter,
        IterKind::RecvIntoIter,
        IterKind::MapIter,
        IterKind::MapIterMut,
        IterKind::MapKeys,
        IterKind::MapValues,
        IterKind::MapValuesMut,
        IterKind::MapIntoIter,
        IterKind::MapIntoKeys,
        IterKind::MapIntoValues,
        IterKind::SetIter,
        IterKind::SetIntoIter,
        IterKind::ArrayIntoIter,
        IterKind::RepeatN,
        IterKind::ByNext,
        IterKind::Windows,
        IterKind::CharIndices,
        IterKind::Flatten,
        IterKind::OptionIntoIter,
        IterKind::OptionIter,
        IterKind::OptionIterMut,
        IterKind::ResultIter,
        IterKind::ResultIterMut,
        IterKind::ResultIntoIter,
        IterKind::StrSplit,
        IterKind::StrMatches,
        IterKind::EncodeUtf16,
        IterKind::VecDrain,
        IterKind::StringDrain,
        IterKind::RangeFrom,
        IterKind::RangeTo,
        IterKind::RangeFull,
        IterKind::RangeToInclusive,
        IterKind::Chunks,
        IterKind::ChunksMut,
        IterKind::ChunksExact,
        IterKind::ChunksExactMut,
        IterKind::Utf8Chunks,
        IterKind::Utf8Chunk,
        IterKind::TryCollect,
        IterKind::Args,
        IterKind::SliceSplit,
        IterKind::SliceSplitMut,
        IterKind::MapWhile,
        IterKind::MapDrain,
        IterKind::SetDrain,
        IterKind::ChunkBy,
        IterKind::ChunkByMut,
    ];

    /// True for the hashed-collection iterators.
    pub fn is_hash(self) -> bool {
        (IterKind::MapIter as usize..=IterKind::SetIntoIter as usize).contains(&(self as usize)) || self.is_hash_drain()
    }

    /// True for the hash drains: they hold the map's table and give it
    /// back emptied when they drop.
    pub fn is_hash_drain(self) -> bool {
        matches!(self, IterKind::MapDrain | IterKind::SetDrain)
    }

    /// True for the hash iterators that own their table (by-value
    /// `into_iter`, `into_keys`, `into_values`).
    pub fn is_hash_owning(self) -> bool {
        matches!(self, IterKind::MapIntoIter | IterKind::MapIntoKeys | IterKind::MapIntoValues | IterKind::SetIntoIter)
    }

    /// The std module path and struct name: the def hash input and
    /// the printed name.
    pub fn path(self) -> (&'static [&'static str], &'static str) {
        use IterKind as K;
        const ADAPT: &[&str] = &["core", "iter", "adapters"];
        const HMAP: &[&str] = &["std", "collections", "hash", "map"];
        const HSET: &[&str] = &["std", "collections", "hash", "set"];
        match self {
            K::SliceIter => (&["core", "slice"], "Iter"),
            K::SliceIterMut => (&["core", "slice"], "IterMut"),
            K::VecIntoIter => (&["alloc", "vec"], "IntoIter"),
            K::Range => (&["core", "ops"], "Range"),
            K::RangeInclusive => (&["core", "ops"], "RangeInclusive"),
            K::Map => (ADAPT, "Map"),
            K::Filter => (ADAPT, "Filter"),
            K::Enumerate => (ADAPT, "Enumerate"),
            K::Rev => (ADAPT, "Rev"),
            K::Take => (ADAPT, "Take"),
            K::Skip => (ADAPT, "Skip"),
            K::Chain => (ADAPT, "Chain"),
            K::Zip => (ADAPT, "Zip"),
            K::Copied => (ADAPT, "Copied"),
            K::Cloned => (ADAPT, "Cloned"),
            K::FilterMap => (ADAPT, "FilterMap"),
            K::TakeWhile => (ADAPT, "TakeWhile"),
            K::SkipWhile => (ADAPT, "SkipWhile"),
            K::StepBy => (ADAPT, "StepBy"),
            K::Peekable => (ADAPT, "Peekable"),
            K::Inspect => (ADAPT, "Inspect"),
            K::RecvIter => (&["std", "sync", "mpsc"], "Iter"),
            K::RecvTryIter => (&["std", "sync", "mpsc"], "TryIter"),
            K::RecvIntoIter => (&["std", "sync", "mpsc"], "IntoIter"),
            K::MapIter => (HMAP, "Iter"),
            K::MapIterMut => (HMAP, "IterMut"),
            K::MapKeys => (HMAP, "Keys"),
            K::MapValues => (HMAP, "Values"),
            K::MapValuesMut => (HMAP, "ValuesMut"),
            K::MapIntoIter => (HMAP, "IntoIter"),
            K::MapIntoKeys => (HMAP, "IntoKeys"),
            K::MapIntoValues => (HMAP, "IntoValues"),
            K::SetIter => (HSET, "Iter"),
            K::SetIntoIter => (HSET, "IntoIter"),
            K::ArrayIntoIter => (&["core", "array", "iter"], "IntoIter"),
            K::RepeatN => (&["core", "iter", "sources", "repeat_n"], "RepeatN"),
            K::ByNext => (&["core", "iter", "lust"], "ByNext"),
            K::Windows => (&["core", "slice"], "Windows"),
            K::CharIndices => (&["core", "str"], "CharIndices"),
            K::Flatten => (ADAPT, "Flatten"),
            K::OptionIntoIter => (&["core", "option"], "IntoIter"),
            K::OptionIter => (&["core", "option"], "Iter"),
            K::OptionIterMut => (&["core", "option"], "IterMut"),
            K::ResultIter => (&["core", "result"], "Iter"),
            K::ResultIterMut => (&["core", "result"], "IterMut"),
            K::ResultIntoIter => (&["core", "result"], "IntoIter"),
            K::StrSplit => (&["core", "str", "iter"], "Split"),
            K::StrMatches => (&["core", "str", "iter"], "Matches"),
            K::EncodeUtf16 => (&["core", "str"], "EncodeUtf16"),
            K::VecDrain => (&["alloc", "vec"], "Drain"),
            K::StringDrain => (&["alloc", "string"], "Drain"),
            K::RangeFrom => (&["core", "ops"], "RangeFrom"),
            K::RangeTo => (&["core", "ops"], "RangeTo"),
            K::RangeFull => (&["core", "ops"], "RangeFull"),
            K::RangeToInclusive => (&["core", "ops"], "RangeToInclusive"),
            K::Chunks => (&["core", "slice"], "Chunks"),
            K::ChunksMut => (&["core", "slice"], "ChunksMut"),
            K::ChunksExact => (&["core", "slice"], "ChunksExact"),
            K::ChunksExactMut => (&["core", "slice"], "ChunksExactMut"),
            K::Utf8Chunks => (&["core", "str"], "Utf8Chunks"),
            K::Utf8Chunk => (&["core", "str"], "Utf8Chunk"),
            K::TryCollect => (&["core", "iter", "lust"], "TryCollect"),
            K::Args => (&["std", "env"], "Args"),
            K::SliceSplit => (&["core", "slice"], "Split"),
            K::SliceSplitMut => (&["core", "slice"], "SplitMut"),
            K::MapWhile => (ADAPT, "MapWhile"),
            K::MapDrain => (HMAP, "Drain"),
            K::SetDrain => (HSET, "Drain"),
            K::ChunkBy => (&["core", "slice"], "ChunkBy"),
            K::ChunkByMut => (&["core", "slice"], "ChunkByMut"),
        }
    }

    /// True for the std adapters: they wrap a builtin iterator (their
    /// first arg; `Chain` and `Zip` a second one too), and core's
    /// `Iterator` impl for them has where clauses.
    pub fn is_adapter(self) -> bool {
        use IterKind as K;
        matches!(
            self,
            K::Map
                | K::Filter
                | K::Enumerate
                | K::Rev
                | K::Take
                | K::Skip
                | K::Chain
                | K::Zip
                | K::Copied
                | K::Cloned
                | K::FilterMap
                | K::MapWhile
                | K::TakeWhile
                | K::SkipWhile
                | K::StepBy
                | K::Peekable
                | K::Inspect
                | K::Flatten
        )
    }

    /// True for the kinds that are iterators in lust. The four
    /// open-ended range kinds are `RangeBounds` values only.
    pub fn is_iterator(self) -> bool {
        !matches!(
            self,
            IterKind::RangeTo | IterKind::RangeFull | IterKind::RangeToInclusive | IterKind::Utf8Chunk
        )
    }

    /// True for the six `core::ops` range kinds.
    pub fn is_range(self) -> bool {
        matches!(
            self,
            IterKind::Range | IterKind::RangeInclusive | IterKind::RangeFrom | IterKind::RangeTo | IterKind::RangeFull | IterKind::RangeToInclusive
        )
    }

    /// How many type args the builtin type takes (see the kind docs).
    pub fn n_args(self) -> usize {
        use IterKind as K;
        match self {
            K::Map | K::FilterMap | K::MapWhile | K::TryCollect | K::SliceSplit | K::SliceSplitMut => 3,
            K::Flatten => 2,
            K::Filter
            | K::Chain
            | K::Zip
            | K::TakeWhile
            | K::SkipWhile
            | K::Inspect
            | K::Copied
            | K::Cloned
            | K::ArrayIntoIter
            | K::ByNext
            | K::StrSplit
            | K::StrMatches => 2,
            K::MapIter
            | K::MapIterMut
            | K::MapKeys
            | K::MapValues
            | K::MapValuesMut
            | K::MapIntoIter
            | K::MapIntoKeys
            | K::MapIntoValues
            | K::MapDrain
            | K::ChunkBy
            | K::ChunkByMut => 2,
            K::CharIndices | K::StringDrain | K::RangeFull | K::EncodeUtf16 | K::Args => 0,
            _ => 1,
        }
    }

    /// The printed path, as rustc's diagnostics name the type.
    pub fn display(self) -> &'static str {
        use IterKind as K;
        match self {
            K::SliceIter => "std::slice::Iter",
            K::SliceIterMut => "std::slice::IterMut",
            K::VecIntoIter => "std::vec::IntoIter",
            K::Range => "std::ops::Range",
            K::RangeInclusive => "std::ops::RangeInclusive",
            K::Map => "std::iter::Map",
            K::Filter => "std::iter::Filter",
            K::Enumerate => "std::iter::Enumerate",
            K::Rev => "std::iter::Rev",
            K::Take => "std::iter::Take",
            K::Skip => "std::iter::Skip",
            K::Chain => "std::iter::Chain",
            K::Zip => "std::iter::Zip",
            K::Copied => "std::iter::Copied",
            K::Cloned => "std::iter::Cloned",
            K::FilterMap => "std::iter::FilterMap",
            K::TakeWhile => "std::iter::TakeWhile",
            K::SkipWhile => "std::iter::SkipWhile",
            K::StepBy => "std::iter::StepBy",
            K::Peekable => "std::iter::Peekable",
            K::Inspect => "std::iter::Inspect",
            K::RecvIter => "std::sync::mpsc::Iter",
            K::RecvTryIter => "std::sync::mpsc::TryIter",
            K::RecvIntoIter => "std::sync::mpsc::IntoIter",
            K::MapIter => "std::collections::hash_map::Iter",
            K::MapIterMut => "std::collections::hash_map::IterMut",
            K::MapKeys => "std::collections::hash_map::Keys",
            K::MapValues => "std::collections::hash_map::Values",
            K::MapValuesMut => "std::collections::hash_map::ValuesMut",
            K::MapIntoIter => "std::collections::hash_map::IntoIter",
            K::MapIntoKeys => "std::collections::hash_map::IntoKeys",
            K::MapIntoValues => "std::collections::hash_map::IntoValues",
            K::SetIter => "std::collections::hash_set::Iter",
            K::SetIntoIter => "std::collections::hash_set::IntoIter",
            K::ArrayIntoIter => "std::array::IntoIter",
            K::RepeatN => "std::iter::RepeatN",
            K::ByNext => "",
            K::Windows => "std::slice::Windows",
            K::CharIndices => "std::str::CharIndices",
            K::Flatten => "std::iter::Flatten",
            K::OptionIntoIter => "std::option::IntoIter",
            K::OptionIter => "std::option::Iter",
            K::OptionIterMut => "std::option::IterMut",
            K::ResultIter => "std::result::Iter",
            K::ResultIterMut => "std::result::IterMut",
            K::ResultIntoIter => "std::result::IntoIter",
            K::StrSplit => "std::str::Split",
            K::StrMatches => "std::str::Matches",
            K::EncodeUtf16 => "std::str::EncodeUtf16",
            K::VecDrain => "std::vec::Drain",
            K::StringDrain => "std::string::Drain",
            K::RangeFrom => "std::ops::RangeFrom",
            K::RangeTo => "std::ops::RangeTo",
            K::RangeFull => "std::ops::RangeFull",
            K::RangeToInclusive => "std::ops::RangeToInclusive",
            K::Chunks => "std::slice::Chunks",
            K::ChunksMut => "std::slice::ChunksMut",
            K::ChunksExact => "std::slice::ChunksExact",
            K::ChunksExactMut => "std::slice::ChunksExactMut",
            K::Utf8Chunks => "std::str::Utf8Chunks",
            K::Utf8Chunk => "std::str::Utf8Chunk",
            K::TryCollect => "",
            K::Args => "std::env::Args",
            K::SliceSplit => "std::slice::Split",
            K::SliceSplitMut => "std::slice::SplitMut",
            K::MapWhile => "std::iter::MapWhile",
            K::MapDrain => "std::collections::hash_map::Drain",
            K::SetDrain => "std::collections::hash_set::Drain",
            K::ChunkBy => "std::slice::ChunkBy",
            K::ChunkByMut => "std::slice::ChunkByMut",
        }
    }

    /// How many leading args the printed name shows (the item args
    /// lust adds stay hidden).
    pub fn shown_args(self) -> usize {
        match self {
            IterKind::Map | IterKind::FilterMap | IterKind::MapWhile | IterKind::SliceSplit | IterKind::SliceSplitMut => 2,
            IterKind::Copied | IterKind::Cloned | IterKind::ArrayIntoIter | IterKind::Flatten | IterKind::StrSplit | IterKind::StrMatches => 1,
            // The `u8` arg of the UTF-8 kinds is lust's own marker.
            IterKind::Utf8Chunks | IterKind::Utf8Chunk => 0,
            k => k.n_args(),
        }
    }

    /// The stored fields, in std declaration order. `vec::IntoIter`
    /// is (buffer, front, back): the buffer's own length is never
    /// changed, and elements in `front..back` are the ones still owned.
    pub fn fields(self) -> &'static [Field] {
        use Field as F;
        use IterKind as K;
        match self {
            K::SliceIter => &[F::SliceRef(0, false)],
            K::SliceIterMut => &[F::SliceRef(0, true)],
            K::VecIntoIter => &[F::VecArg(0), F::Usize, F::Usize],
            K::Range => &[F::Arg(0), F::Arg(0)],
            K::RangeInclusive => &[F::Arg(0), F::Arg(0), F::Bool],
            K::Map | K::Filter | K::FilterMap | K::MapWhile | K::Inspect => &[F::Arg(0), F::Arg(1)],
            // `index` and `len` serve the random-access form only.
            K::Zip => &[F::Arg(0), F::Arg(1), F::Usize, F::Usize],
            K::Enumerate | K::Take | K::Skip => &[F::Arg(0), F::Usize],
            K::Rev | K::Copied | K::Cloned => &[F::Arg(0)],
            K::Chain => &[F::OptArg(0), F::OptArg(1)],
            K::TakeWhile | K::SkipWhile => &[F::Arg(0), F::Bool, F::Arg(1)],
            K::StepBy => &[F::Arg(0), F::Usize, F::Bool],
            K::Peekable => &[F::Arg(0), F::PeekSlot(0)],
            K::RecvIter | K::RecvTryIter => &[F::Recv(0, false)],
            K::RecvIntoIter => &[F::Recv(0, true)],
            // The hash core's iterator state (cgen/hash.rs), never a place.
            K::MapIter
            | K::MapIterMut
            | K::MapKeys
            | K::MapValues
            | K::MapValuesMut
            | K::MapIntoIter
            | K::MapIntoKeys
            | K::MapIntoValues
            | K::SetIter
            | K::SetIntoIter
            | K::MapDrain
            | K::SetDrain => &[],
            // (array, front, back): elements in `front..back` remain.
            K::ArrayIntoIter => &[F::Arg(1), F::Usize, F::Usize],
            // (element, count left): the element is `None` exactly when
            // the count is 0.
            K::RepeatN => &[F::OptArg(0), F::Usize],
            K::ByNext => &[F::Arg(0)],
            K::Windows => &[F::SliceRef(0, false), F::Usize],
            K::CharIndices => &[F::Chars, F::Usize],
            K::Flatten => &[F::OptArg(0), F::OptArg(1), F::OptArg(1)],
            K::OptionIntoIter | K::ResultIntoIter => &[F::OptArg(0)],
            K::OptionIter | K::ResultIter => &[F::OptRef(0, false)],
            K::OptionIterMut | K::ResultIterMut => &[F::OptRef(0, true)],
            K::StrSplit | K::StrMatches => &[F::Str, F::Usize, F::Usize, F::Usize, F::Usize, F::Usize, F::Usize, F::Arg(0)],
            K::EncodeUtf16 => &[F::Chars, F::Usize],
            // The drains: the borrowed container, then start, front,
            // back, tail start, and tail length, as indexes into the
            // container's buffer (std's Drain bookkeeping).
            K::VecDrain => &[F::VecRefMut(0), F::Usize, F::Usize, F::Usize, F::Usize, F::Usize],
            K::StringDrain => &[F::StringRefMut, F::Usize, F::Usize, F::Usize, F::Usize, F::Usize],
            K::Chunks => &[F::SliceRef(0, false), F::Usize],
            K::ChunksMut => &[F::SliceRef(0, true), F::Usize],
            K::ChunksExact => &[F::SliceRef(0, false), F::SliceRef(0, false), F::Usize],
            K::ChunksExactMut => &[F::SliceRef(0, true), F::SliceRef(0, true), F::Usize],
            K::Utf8Chunks => &[F::SliceRef(0, false)],
            K::Utf8Chunk => &[F::Str, F::SliceRef(0, false)],
            K::RangeFrom | K::RangeTo | K::RangeToInclusive => &[F::Arg(0)],
            // No stored value; one dummy field, as `RangeInclusive`
            // stores its flag.
            K::RangeFull => &[F::Bool],
            K::TryCollect => &[F::Arg(0), F::OptArg(1)],
            K::Args => &[F::Usize, F::Usize],
            K::SliceSplit => &[F::SliceRef(0, false), F::Arg(1), F::Bool, F::Usize],
            K::SliceSplitMut => &[F::SliceRef(0, true), F::Arg(1), F::Bool, F::Usize],
            K::ChunkBy => &[F::SliceRef(0, false), F::Arg(1)],
            K::ChunkByMut => &[F::SliceRef(0, true), F::Arg(1)],
        }
    }
}

/// The def of one builtin iterator struct (see `BuiltinDefs::iters`).
pub(super) fn iter_def(k: IterKind) -> DefId {
    let (module, name) = k.path();
    builtin_def(module, name)
}

/// The kind and type args of a builtin iterator type.
pub fn iter_parts(tys: &impl TyIntern, t: Ty) -> Option<(IterKind, Vec<Ty>)> {
    let Some(BuiltinTy::Iter(k)) = tys.builtin_of(t) else { return None };
    let (_, a) = tys.adt_parts(t)?;
    Some((k, tys.args(a).iter().map(|g| g.as_ty().unwrap_or(ERROR)).collect()))
}

/// A builtin iterator type from its kind and args.
pub fn mk_iter(tys: &mut impl TyIntern, k: IterKind, args: &[Ty]) -> Ty {
    debug_assert_eq!(args.len(), k.n_args());
    let def = tys.builtins().iters[k as usize];
    let a: Vec<GenericArg> = args.iter().map(|&t| GenericArg::Ty(t)).collect();
    tys.mk_adt(def, &a)
}

/// True when a builtin iterator type holds a `ByNext` (a user iterator
/// or a type parameter) at any depth of its iterator args.
pub fn has_by_next(tys: &impl TyIntern, t: Ty) -> bool {
    match iter_parts(tys, t) {
        Some((IterKind::ByNext, _)) => true,
        Some((_, a)) => a.iter().any(|&x| is_iter(tys, x) && has_by_next(tys, x)),
        None => false,
    }
}

/// True for the types `item_ty` answers: builtin iterators and `Chars`.
pub fn is_iter(tys: &impl TyIntern, t: Ty) -> bool {
    match tys.builtin_of(t) {
        Some(BuiltinTy::Iter(k)) => k.is_iterator(),
        Some(BuiltinTy::Chars) => true,
        _ => false,
    }
}

/// The item type of a builtin iterator type or `Chars`.
pub fn item_ty(tys: &mut impl TyIntern, t: Ty) -> Option<Ty> {
    use IterKind as K;
    if tys.is_chars_ty(t) {
        return Some(CHAR);
    }
    let (k, a) = iter_parts(tys, t)?;
    if !k.is_iterator() {
        return None;
    }
    Some(match k {
        K::SliceIter => tys.mk_ref(false, a[0]),
        K::SliceIterMut => tys.mk_ref(true, a[0]),
        K::VecIntoIter | K::Range | K::RangeInclusive | K::RangeFrom | K::RecvIter | K::RecvTryIter | K::RecvIntoIter | K::VecDrain => a[0],
        K::StringDrain => CHAR,
        K::RangeTo | K::RangeFull | K::RangeToInclusive | K::Utf8Chunk => return None,
        K::Map | K::FilterMap | K::MapWhile => a[2],
        K::Filter | K::Rev | K::Take | K::Skip | K::Chain | K::TakeWhile | K::SkipWhile | K::StepBy | K::Peekable | K::Inspect => {
            item_ty(tys, a[0])?
        }
        K::Enumerate => {
            let inner = item_ty(tys, a[0])?;
            let ts = tys.intern_tys(&[USIZE, inner]);
            tys.intern(TyKind::Tuple(ts))
        }
        K::Zip => {
            let x = item_ty(tys, a[0])?;
            let y = item_ty(tys, a[1])?;
            let ts = tys.intern_tys(&[x, y]);
            tys.intern(TyKind::Tuple(ts))
        }
        K::Copied | K::Cloned => a[1],
        K::MapIter | K::MapIterMut => {
            let kr = tys.mk_ref(false, a[0]);
            let vr = tys.mk_ref(k == K::MapIterMut, a[1]);
            let ts = tys.intern_tys(&[kr, vr]);
            tys.intern(TyKind::Tuple(ts))
        }
        K::MapKeys | K::SetIter => tys.mk_ref(false, a[0]),
        K::MapValues => tys.mk_ref(false, a[1]),
        K::MapValuesMut => tys.mk_ref(true, a[1]),
        K::MapIntoIter | K::MapDrain => {
            let ts = tys.intern_tys(&[a[0], a[1]]);
            tys.intern(TyKind::Tuple(ts))
        }
        K::MapIntoKeys | K::SetIntoIter | K::SetDrain => a[0],
        K::MapIntoValues => a[1],
        K::ArrayIntoIter | K::RepeatN => a[0],
        K::ByNext => a[1],
        K::Windows => {
            let s = tys.intern(TyKind::Slice(a[0]));
            tys.mk_ref(false, s)
        }
        K::Chunks | K::ChunksExact => {
            let s = tys.intern(TyKind::Slice(a[0]));
            tys.mk_ref(false, s)
        }
        K::ChunksMut | K::ChunksExactMut => {
            let s = tys.intern(TyKind::Slice(a[0]));
            tys.mk_ref(true, s)
        }
        K::Utf8Chunks => mk_iter(tys, K::Utf8Chunk, &[a[0]]),
        K::CharIndices => {
            let ts = tys.intern_tys(&[USIZE, CHAR]);
            tys.intern(TyKind::Tuple(ts))
        }
        K::Flatten => item_ty(tys, a[1])?,
        K::OptionIntoIter | K::ResultIntoIter => a[0],
        K::OptionIter | K::ResultIter => tys.mk_ref(false, a[0]),
        K::OptionIterMut | K::ResultIterMut => tys.mk_ref(true, a[0]),
        K::StrSplit => REF_STR,
        K::StrMatches => match matches_mode(tys, a[1]) {
            Some(m) if m.indexed() => {
                let ts = tys.intern_tys(&[USIZE, REF_STR]);
                tys.intern(TyKind::Tuple(ts))
            }
            _ => REF_STR,
        },
        K::EncodeUtf16 => int(IntTy::U16),
        K::TryCollect => a[2],
        K::Args => STRING,
        K::SliceSplit | K::SliceSplitMut => {
            let s = tys.intern(TyKind::Slice(a[0]));
            tys.mk_ref(k == K::SliceSplitMut, s)
        }
        K::ChunkBy | K::ChunkByMut => {
            let s = tys.intern(TyKind::Slice(a[0]));
            tys.mk_ref(k == K::ChunkByMut, s)
        }
    })
}

/// The hash iterator a `for` loop over `t` runs (W-STD-B): a map or set
/// by value (`into_iter`), `&` of one (`iter`), or `&mut` of a map
/// (`iter_mut`). `Some(None)` for `&mut` of a set (no `IntoIterator`
/// impl), None when `t` is none of these.
pub fn hash_for_iter(tys: &mut impl TyIntern, t: Ty) -> Option<Option<Ty>> {
    use crate::ty::hash::{HashTy, hash_parts};
    let (m, inner) = match tys.kind(t) {
        TyKind::Ref(_, m, inner) => (Some(m.is_mut()), inner),
        _ => (None, t),
    };
    let (k, a) = hash_parts(tys, inner)?;
    let (kind, args) = match (k, m) {
        (HashTy::HashMap, None) => (IterKind::MapIntoIter, vec![a[0], a[1]]),
        (HashTy::HashMap, Some(false)) => (IterKind::MapIter, vec![a[0], a[1]]),
        (HashTy::HashMap, Some(true)) => (IterKind::MapIterMut, vec![a[0], a[1]]),
        (HashTy::HashSet, None) => (IterKind::SetIntoIter, vec![a[0]]),
        (HashTy::HashSet, Some(false)) => (IterKind::SetIter, vec![a[0]]),
        (HashTy::HashSet, Some(true)) => return Some(None),
        _ => return None,
    };
    Some(Some(mk_iter(tys, kind, &args)))
}

/// The std iterator a full std type path names (W-STD-B):
/// `vec::IntoIter<T>`, the hash `IntoIter`, `IntoKeys`, and
/// `IntoValues`, and the `option` and `result` iterators (their one
/// lifetime is erased, as `Cow`'s is). The other borrowing iterators
/// (`slice::Iter<'a, T>`) stay unknown types.
pub fn owning_by_path(path: &[&str]) -> Option<IterKind> {
    Some(match path {
        ["std" | "alloc", "vec", "IntoIter"] => IterKind::VecIntoIter,
        ["std" | "core", "option", "Iter"] => IterKind::OptionIter,
        ["std" | "core", "option", "IterMut"] => IterKind::OptionIterMut,
        ["std" | "core", "option", "IntoIter"] => IterKind::OptionIntoIter,
        ["std" | "core", "result", "Iter"] => IterKind::ResultIter,
        ["std" | "core", "result", "IterMut"] => IterKind::ResultIterMut,
        ["std" | "core", "result", "IntoIter"] => IterKind::ResultIntoIter,
        ["std", "collections", "hash_map", "IntoIter"] => IterKind::MapIntoIter,
        ["std", "collections", "hash_map", "IntoKeys"] => IterKind::MapIntoKeys,
        ["std", "collections", "hash_map", "IntoValues"] => IterKind::MapIntoValues,
        ["std", "collections", "hash_set", "IntoIter"] => IterKind::SetIntoIter,
        _ => return None,
    })
}

/// The iterator an `Option` or `Result` turns into through
/// `IntoIterator` (fx-option-rest): by value its `IntoIter`, by
/// `&` its `Iter`, by `&mut` its `IterMut`. None for other types.
pub fn opt_res_iter(tys: &mut impl TyIntern, t: Ty) -> Option<Ty> {
    let (m, inner) = match tys.kind(t) {
        TyKind::Ref(_, m, inner) => (Some(m.is_mut()), inner),
        _ => (None, t),
    };
    let (k, el) = if let Some(el) = tys.option_elem(inner) {
        (match m { None => IterKind::OptionIntoIter, Some(false) => IterKind::OptionIter, Some(true) => IterKind::OptionIterMut }, el)
    } else if let Some((el, _)) = tys.result_args(inner) {
        (match m { None => IterKind::ResultIntoIter, Some(false) => IterKind::ResultIter, Some(true) => IterKind::ResultIterMut }, el)
    } else {
        return None;
    };
    Some(mk_iter(tys, k, &[el]))
}

/// True when `k` is one of the six `option` and `result` iterators.
pub fn is_opt_res(k: IterKind) -> bool {
    matches!(
        k,
        IterKind::OptionIter
            | IterKind::OptionIterMut
            | IterKind::OptionIntoIter
            | IterKind::ResultIter
            | IterKind::ResultIterMut
            | IterKind::ResultIntoIter
    )
}

/// The iterator an `IntoIterator` argument (`extend`) turns into: a
/// builtin iterator as itself, a Vec by value as `vec::IntoIter`, `&X`
/// or `&mut X` over a Vec, slice, or array as a slice iterator, and a
/// map or set as its `hash_for_iter` iterator. None for other types.
pub fn as_iter_ty(tys: &mut impl TyIntern, t: Ty) -> Option<Ty> {
    if is_iter(tys, t) {
        return Some(t);
    }
    // An array by value (`array::IntoIter`, fx-array-iter).
    if let TyKind::Array(e, _) = tys.kind(t) {
        return Some(mk_iter(tys, IterKind::ArrayIntoIter, &[e, t]));
    }
    if let Some(it) = hash_for_iter(tys, t) {
        return it;
    }
    if let Some(it) = opt_res_iter(tys, t) {
        return Some(it);
    }
    if let Some(el) = tys.vec_elem(t) {
        return Some(mk_iter(tys, IterKind::VecIntoIter, &[el]));
    }
    let TyKind::Ref(_, m, inner) = tys.kind(t) else { return None };
    let el = match tys.kind(inner) {
        TyKind::Slice(el) | TyKind::Array(el, _) => el,
        _ => tys.vec_elem(inner)?,
    };
    Some(mk_iter(tys, if m.is_mut() { IterKind::SliceIterMut } else { IterKind::SliceIter }, &[el]))
}

/// The semantic type of one stored field.
pub fn field_ty(tys: &mut impl TyIntern, f: Field, args: &[Ty]) -> Ty {
    match f {
        Field::Arg(i) => args[i as usize],
        Field::Usize => USIZE,
        Field::Bool => BOOL,
        Field::OptArg(i) => tys.mk_option(args[i as usize]),
        Field::OptRef(i, m) => {
            let r = tys.mk_ref(m, args[i as usize]);
            tys.mk_option(r)
        }
        Field::SliceRef(i, m) => {
            let s = tys.intern(TyKind::Slice(args[i as usize]));
            tys.mk_ref(m, s)
        }
        Field::VecArg(i) => tys.mk_vec(args[i as usize]),
        Field::VecRefMut(i) => {
            let v = tys.mk_vec(args[i as usize]);
            tys.mk_ref(true, v)
        }
        Field::StringRefMut => tys.mk_ref(true, STRING),
        Field::PeekSlot(i) => {
            let item = item_ty(tys, args[i as usize]).unwrap_or(ERROR);
            let inner = tys.mk_option(item);
            tys.mk_option(inner)
        }
        Field::Recv(i, owned) => {
            let r = super::sync::sync_ty(tys, super::sync::SyncTy::Receiver, &[args[i as usize]]);
            if owned { r } else { tys.mk_ref(false, r) }
        }
        Field::Chars => CHARS,
        Field::Str => REF_STR,
    }
}

/// The stored field types of a builtin iterator type, in order.
pub fn field_tys(tys: &mut impl TyIntern, t: Ty) -> Option<Vec<Ty>> {
    let (k, a) = iter_parts(tys, t)?;
    Some(k.fields().iter().map(|&f| field_ty(tys, f, &a)).collect())
}

/// True if a builtin iterator type can hold a reference: the slice
/// iterators always do; every other kind does when one of its args
/// does (an arg covers every field and the item, see the kind docs).
pub fn iter_holds_ref(k: IterKind, args: &[Ty], arg_holds: &dyn Fn(Ty) -> bool) -> bool {
    matches!(
        k,
        IterKind::SliceIter
            | IterKind::SliceIterMut
            | IterKind::Windows
            | IterKind::Chunks
            | IterKind::ChunksMut
            | IterKind::ChunksExact
            | IterKind::ChunksExactMut
            | IterKind::Utf8Chunks
            | IterKind::Utf8Chunk
            | IterKind::CharIndices
            | IterKind::StrSplit
            | IterKind::MapIter
            | IterKind::MapIterMut
            | IterKind::MapKeys
            | IterKind::MapValues
            | IterKind::MapValuesMut
            | IterKind::SetIter
            | IterKind::VecDrain
            | IterKind::StringDrain
            | IterKind::OptionIter
            | IterKind::OptionIterMut
            | IterKind::ResultIter
            | IterKind::ResultIterMut
    ) || args.iter().any(|&a| arg_holds(a))
}
