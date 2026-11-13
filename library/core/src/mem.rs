// `core::mem` overlay (docs/ARCH.md section 3.1; TRIAGE T5, R2):
// `MaybeUninit`'s Debug, `ManuallyDrop`, and `Discriminant`. `MaybeUninit` itself is
// a synthetic std Adt (check/collect.rs registers it; `uninit`,
// `new`, `write`, and the pointer accessors are compiler builtins in
// check/iter.rs), so only its impl lives here. `core::mem::X` and `std::mem::X`
// walk into this module only for the names it defines
// (`STD_MODULES` in src/stdmods.rs).
//
// Lust std source, adapted from rust-lang/rust 1.99
// `core/src/mem/maybe_uninit.rs`: Debug prints the type name and
// never the (possibly uninitialized) value.

impl<T> core::fmt::Debug for core::mem::MaybeUninit<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "MaybeUninit<{}>", core::any::type_name::<T>())
    }
}

// Lust std source, adapted from rust-lang/rust 1.99
// `core/src/mem/manually_drop.rs` and `maybe_dangling.rs`. The
// compiler knows `ManuallyDrop` by its path
// (`Program::is_manually_drop_adt`): it has no drop glue, so the
// value inside is never dropped, and layout queries see it as
// repr(transparent) over `T` with `T`'s niche. The unsafe fns
// (`drop`, `take`) are not here: lust has no `unsafe`. No
// `T: ?Sized` yet.

/// A wrapper that stops the compiler from dropping its value. Core writes
/// `PartialEq`, `Eq`, `PartialOrd`, `Ord`, and `Hash` by hand over the one
/// field (and `StructuralPartialEq`); derives over the one field are the
/// same impls, and lust's sort, map keys, and const patterns take them.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ManuallyDrop<T> {
    value: MaybeDangling<T>,
}

/// Core's (unstable) field type of `ManuallyDrop`. Private here, so
/// user code cannot name it; it shows in `ManuallyDrop`'s Debug text
/// as in core (`ManuallyDrop { value: MaybeDangling(5) }`).
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
struct MaybeDangling<T>(T);

impl<T> ManuallyDrop<T> {
    /// Wraps `value`, so it is not dropped.
    pub const fn new(value: T) -> ManuallyDrop<T> {
        ManuallyDrop { value: MaybeDangling(value) }
    }

    /// Takes the value out, so it is dropped again as usual.
    pub const fn into_inner(slot: ManuallyDrop<T>) -> T {
        slot.value.0
    }
}

impl<T> core::ops::Deref for ManuallyDrop<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value.0
    }
}

impl<T> core::ops::DerefMut for ManuallyDrop<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value.0
    }
}

// Lust std source, adapted from rust-lang/rust 1.99
// `core/src/mem/mod.rs`: `Discriminant<T>` and `discriminant`. Core keeps
// `<T as DiscriminantKind>::Discriminant`, the enum's `repr` integer
// type (`isize` without one, `u8` for a type that is not an enum); lust
// keeps the value as an i128 and the type as `kind`, a code (0 to 11:
// i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize, src/cgen.rs), so
// Debug and Hash format and hash exactly that type. Std module types
// have no layout queries (`Program::std_module_def`), so the extra
// field never shows in `size_of`.

/// An opaque value of an enum variant's discriminant, for comparison.
pub struct Discriminant<T> {
    value: i128,
    kind: u8,
    _t: core::marker::PhantomData<fn() -> T>,
}

impl<T> Copy for Discriminant<T> {}

impl<T> Clone for Discriminant<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> PartialEq for Discriminant<T> {
    fn eq(&self, rhs: &Self) -> bool {
        self.value == rhs.value
    }
}

impl<T> Eq for Discriminant<T> {}

impl<T> core::hash::Hash for Discriminant<T> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        match self.kind {
            0 => (self.value as i8).hash(state),
            1 => (self.value as i16).hash(state),
            2 => (self.value as i32).hash(state),
            3 => (self.value as i64).hash(state),
            4 => self.value.hash(state),
            5 => (self.value as isize).hash(state),
            6 => (self.value as u8).hash(state),
            7 => (self.value as u16).hash(state),
            8 => (self.value as u32).hash(state),
            9 => (self.value as u64).hash(state),
            10 => (self.value as u128).hash(state),
            _ => (self.value as usize).hash(state),
        }
    }
}

impl<T> core::fmt::Debug for Discriminant<T> {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut t = fmt.debug_tuple("Discriminant");
        match self.kind {
            0 => t.field(&(self.value as i8)),
            1 => t.field(&(self.value as i16)),
            2 => t.field(&(self.value as i32)),
            3 => t.field(&(self.value as i64)),
            4 => t.field(&self.value),
            5 => t.field(&(self.value as isize)),
            6 => t.field(&(self.value as u8)),
            7 => t.field(&(self.value as u16)),
            8 => t.field(&(self.value as u32)),
            9 => t.field(&(self.value as u64)),
            10 => t.field(&(self.value as u128)),
            _ => t.field(&(self.value as usize)),
        };
        t.finish()
    }
}

/// The value that identifies the enum variant of `v`.
pub const fn discriminant<T>(v: &T) -> Discriminant<T> {
    Discriminant {
        value: core::intrinsics::discriminant_value(v),
        kind: core::intrinsics::discriminant_kind(v),
        _t: core::marker::PhantomData,
    }
}
