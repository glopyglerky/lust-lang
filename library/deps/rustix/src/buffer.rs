// `rustix::buffer`: the sealed `Buffer` trait of `read`. Upstream's
// counting forms are here (`&mut [T]`, `&mut [T; N]`, `&mut Vec<T>`,
// whose upstream `Output` is `usize`, so `read` returns a count); the
// `MaybeUninit` and `SpareCapacity` forms are not. The sealed hook is a
// `__lust_` name user code cannot call (check/platform.rs); upstream's
// `Output`, `parts_mut`, and `assume_init` are not here.

/// A buffer `read` fills.
pub trait Buffer<T>: private::Sealed<T> {}

impl<T> Buffer<T> for &mut [T] {}
impl<T, const N: usize> Buffer<T> for &mut [T; N] {}
#[cfg(feature = "alloc")]
impl<T> Buffer<T> for &mut Vec<T> {}

pub(crate) mod private {
    /// The sealed half: the slice to fill.
    pub trait Sealed<T> {
        /// The initialized slice to fill.
        fn __lust_slice(&mut self) -> &mut [T];
    }

    impl<T> Sealed<T> for &mut [T] {
        fn __lust_slice(&mut self) -> &mut [T] {
            &mut **self
        }
    }

    impl<T, const N: usize> Sealed<T> for &mut [T; N] {
        fn __lust_slice(&mut self) -> &mut [T] {
            &mut self[..]
        }
    }

    #[cfg(feature = "alloc")]
    impl<T> Sealed<T> for &mut Vec<T> {
        fn __lust_slice(&mut self) -> &mut [T] {
            &mut self[..]
        }
    }
}
