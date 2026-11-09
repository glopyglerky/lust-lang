// `std::alloc` overlay (docs/ARCH.md section 3.1): `System`, `Layout`,
// `LayoutError`, and `handle_alloc_error`. `std::alloc::X` walks into this module only for
// the names it defines (`STD_MODULES` in src/stdmods.rs); `GlobalAlloc`
// stays a compiler-rejected path, as before.
//
// A `#[global_allocator]` static of this type is accepted and the
// lust runtime keeps its own allocator: allocation strategy does
// not change a program's output or exit code. Collect enforces
// the known-allocator rule (check/collect.rs).

/// The system allocator. Zero-sized, as in std.
#[derive(Copy, Clone, Default, Debug)]
pub struct System;

/// Size and alignment of a block of memory (core::alloc::Layout). The
/// subset ts-rust uses: `new`, `for_value`, `array`, `from_size_align`,
/// `size`, `align`, and std's derives and Debug form (`Layout { size: 8,
/// align: 8 (1 << 3) }`). Hash is left out: std hashes the align through a
/// private enum, which this struct does not model.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Layout {
    size: usize,
    align: usize,
}

/// The error of `Layout::array` and `Layout::from_size_align`. std's is a
/// `#[non_exhaustive]` unit struct, so user code cannot build one; the
/// private field keeps that here.
#[derive(Clone, PartialEq, Eq)]
pub struct LayoutError {
    _priv: (),
}

impl core::fmt::Debug for LayoutError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("LayoutError")
    }
}

impl core::fmt::Display for LayoutError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("invalid parameters to Layout::from_size_align")
    }
}

/// std's `Alignment` Debug form: `8 (1 << 3)`.
struct AlignShown(usize);

impl core::fmt::Debug for AlignShown {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?} (1 << {:?})", self.0, self.0.trailing_zeros())
    }
}

impl core::fmt::Debug for Layout {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Layout").field("size", &self.size).field("align", &AlignShown(self.align)).finish()
    }
}

impl Layout {
    /// The largest size a layout of alignment `align` may have:
    /// `isize::MAX` rounded down to a multiple of `align`.
    const fn max_size_for_align(align: usize) -> usize {
        isize::MAX as usize - (align - 1)
    }

    /// A layout of `size` bytes at `align`, which must be a power of two.
    pub const fn from_size_align(size: usize, align: usize) -> Result<Layout, LayoutError> {
        if !align.is_power_of_two() || size > Layout::max_size_for_align(align) {
            return Err(LayoutError { _priv: () });
        }
        Ok(Layout { size, align })
    }

    /// The layout of a `T`.
    pub const fn new<T>() -> Layout {
        Layout { size: core::mem::size_of::<T>(), align: core::mem::align_of::<T>() }
    }

    /// The layout of the value `t` points at.
    pub fn for_value<T: ?Sized>(t: &T) -> Layout {
        Layout { size: core::mem::size_of_val(t), align: core::mem::align_of_val(t) }
    }

    /// The layout of `[T; n]`, or an error when its size passes
    /// `isize::MAX` rounded to T's alignment.
    pub const fn array<T>(n: usize) -> Result<Layout, LayoutError> {
        let size = core::mem::size_of::<T>();
        let align = core::mem::align_of::<T>();
        if size != 0 && n > Layout::max_size_for_align(align) / size {
            return Err(LayoutError { _priv: () });
        }
        Ok(Layout { size: size * n, align })
    }

    /// The size in bytes.
    pub const fn size(&self) -> usize {
        self.size
    }

    /// The alignment in bytes.
    pub const fn align(&self) -> usize {
        self.align
    }
}

/// std's `handle_alloc_error` with the default alloc error hook: prints
/// `memory allocation of N bytes failed` (and the RUST_BACKTRACE note)
/// and aborts.
pub fn handle_alloc_error(layout: Layout) -> ! {
    core::intrinsics::alloc_error(layout.size)
}
