// `core::cmp` overlay (docs/ARCH.md section 3.1; W-STD-C): `Reverse`.
// The other `core::cmp` items (Ordering, the comparison traits, max,
// min) stay compiler-known; `std::cmp::X` and `core::cmp::X` walk into
// this module only for the names it defines (`STD_MODULES` in
// src/stdmods.rs).
//
// As core 1.99 (core/src/cmp.rs): derived Copy, Debug, Hash, PartialEq,
// Eq, and Default; PartialOrd and Ord compare the other way round, with
// each of `lt`, `le`, `gt`, and `ge` written out (a user `PartialOrd`
// on `T` that overrides them is called the same way); Clone forwards
// `clone_from`.

use core::cmp::Ordering;

/// A wrapper that reverses the order of `T`.
#[derive(Copy, Debug, Hash, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct Reverse<T>(pub T);

impl<T: PartialOrd> PartialOrd for Reverse<T> {
    fn partial_cmp(&self, other: &Reverse<T>) -> Option<Ordering> {
        other.0.partial_cmp(&self.0)
    }

    fn lt(&self, other: &Self) -> bool {
        other.0 < self.0
    }

    fn le(&self, other: &Self) -> bool {
        other.0 <= self.0
    }

    fn gt(&self, other: &Self) -> bool {
        other.0 > self.0
    }

    fn ge(&self, other: &Self) -> bool {
        other.0 >= self.0
    }
}

impl<T: Ord> Ord for Reverse<T> {
    fn cmp(&self, other: &Reverse<T>) -> Ordering {
        other.0.cmp(&self.0)
    }
}

impl<T: Clone> Clone for Reverse<T> {
    fn clone(&self) -> Reverse<T> {
        Reverse(self.0.clone())
    }

    fn clone_from(&mut self, source: &Self) {
        self.0.clone_from(&source.0)
    }
}
