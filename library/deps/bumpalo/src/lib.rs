// Lust adapter for bumpalo 3.20.3 (docs/ARCH.md section 3.6): the `Bump`
// subset that ts-rust's leak arena uses. Each value lives in its own
// leaked box, so it never drops, as in a Bump. The chunk bookkeeping is
// upstream's in integers: chunk sizes (power-of-two and page rounding,
// doubling on growth), the bump position, `allocated_bytes`,
// `chunk_capacity`, allocation limits, and which allocations fail are
// the same. A chunk counts as aligned to its own alignment; upstream's
// answer for an allocation with a larger alignment than its chunk's
// depends on malloc's address, so either answer is one rustc can give.
// Not here (they reject at the use): MIN_ALIGN other than 1, reset,
// iter_allocated_chunks, alloc_str, alloc_slice_clone, the collections
// and boxed features, and the Allocator impls.

use core::cell::Cell;
use core::fmt;

/// Upstream's chunk footer: 48 bytes, 16-aligned.
const FOOTER_SIZE: usize = 48;
const CHUNK_ALIGN: usize = 16;
/// Malloc's overhead plus the footer, rounded to CHUNK_ALIGN.
const OVERHEAD: usize = 64;
const DEFAULT_CHUNK_SIZE_WITHOUT_FOOTER: usize = 512 - OVERHEAD;
const TYPICAL_PAGE_SIZE: usize = 0x1000;

/// An allocation failed (an allocation limit, or no room in the chunk).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AllocErr;

impl fmt::Display for AllocErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("memory allocation failed")
    }
}

/// A bump arena. The values it hands out live until the program ends.
pub struct Bump {
    /// The current chunk's size without its footer (0 for the empty chunk).
    chunk_size: Cell<usize>,
    /// The bump position: bytes left below it in the current chunk.
    ptr: Cell<usize>,
    /// The total chunk sizes so far, without footers.
    allocated: Cell<usize>,
    allocation_limit: Cell<Option<usize>>,
    /// Upstream's `Bump` holds a `NonNull` chunk pointer, so it is not
    /// `UnwindSafe`; `&mut ()` gives the same auto traits (Send stays).
    _unwind: core::marker::PhantomData<&'static mut ()>,
}

fn round_up(n: usize, align: usize) -> Option<usize> {
    Some(n.checked_add(align - 1)? & !(align - 1))
}

fn next_power_of_two(n: usize) -> usize {
    let mut p: usize = 1;
    while p < n {
        p *= 2;
    }
    p
}

/// The size without footer of a new chunk for `size` (None: the
/// default) that must hold an allocation of `layout_size`, `align`.
fn new_chunk_size(size: Option<usize>, layout_size: usize, layout_align: usize) -> Option<usize> {
    let align = if layout_align > CHUNK_ALIGN { layout_align } else { CHUNK_ALIGN };
    let mut n = match size {
        Some(s) => s,
        None => DEFAULT_CHUNK_SIZE_WITHOUT_FOOTER,
    };
    let requested = match round_up(layout_size, align) {
        Some(r) => r,
        None => panic!("requested allocation size overflowed"),
    };
    if requested > n {
        n = requested;
    }
    if n < TYPICAL_PAGE_SIZE {
        n = next_power_of_two(n + OVERHEAD) - OVERHEAD;
    } else {
        n = round_up(n + OVERHEAD, TYPICAL_PAGE_SIZE)? - OVERHEAD;
    }
    match n.checked_add(FOOTER_SIZE) {
        Some(_) => Some(n),
        None => panic!("requested allocation size overflowed"),
    }
}

fn oom() -> ! {
    panic!("out of memory")
}

impl Bump {
    pub fn new() -> Bump {
        Bump::with_capacity(0)
    }

    pub fn with_capacity(capacity: usize) -> Bump {
        let b = Bump { chunk_size: Cell::new(0), ptr: Cell::new(0), allocated: Cell::new(0), allocation_limit: Cell::new(None), _unwind: core::marker::PhantomData };
        if capacity > 0 {
            match new_chunk_size(None, capacity, 1) {
                Some(n) => b.start_chunk(n),
                None => oom(),
            }
        }
        b
    }

    fn start_chunk(&self, n: usize) {
        self.chunk_size.set(n);
        self.ptr.set(n);
        self.allocated.set(self.allocated.get() + n);
    }

    pub fn set_allocation_limit(&self, limit: Option<usize>) {
        self.allocation_limit.set(limit);
    }

    pub fn allocation_limit(&self) -> Option<usize> {
        self.allocation_limit.get()
    }

    pub fn allocated_bytes(&self) -> usize {
        self.allocated.get()
    }

    /// The room left in the current chunk.
    pub fn chunk_capacity(&self) -> usize {
        self.ptr.get()
    }

    /// Bumps down for `size` bytes at `align` in the current chunk.
    fn alloc_fast(&self, size: usize, align: usize) -> bool {
        let p = self.ptr.get();
        let aligned_size = match round_up(size, align) {
            Some(s) => s,
            None => return false,
        };
        let aligned_p = p & !(align - 1);
        if aligned_size > aligned_p {
            return false;
        }
        self.ptr.set(aligned_p - aligned_size);
        true
    }

    /// Upstream's slow path: try new chunk sizes, halving from twice the
    /// current one, under the allocation limit.
    fn alloc_slow(&self, size: usize, align: usize) -> bool {
        let remaining = match self.allocation_limit.get() {
            Some(limit) => {
                if self.allocated.get() > limit {
                    None
                } else {
                    Some(limit - self.allocated.get())
                }
            }
            None => None,
        };
        let min_new = if size > DEFAULT_CHUNK_SIZE_WITHOUT_FOOTER { size } else { DEFAULT_CHUNK_SIZE_WITHOUT_FOOTER };
        let mut base = match self.chunk_size.get().checked_mul(2) {
            Some(b) => b,
            None => return false,
        };
        if base < min_new {
            base = min_new;
        }
        loop {
            let bypass = match self.allocation_limit.get() {
                Some(limit) => {
                    size < limit && base >= size && limit < DEFAULT_CHUNK_SIZE_WITHOUT_FOOTER && self.allocated.get() == 0
                }
                None => false,
            };
            if !(base >= min_new || bypass) {
                return false;
            }
            let try_size = base;
            base /= 2;
            // An overflowing size ends the candidates, as upstream's
            // iter::from_fn does at its first None.
            let n = match new_chunk_size(Some(try_size), size, align) {
                Some(n) => n,
                None => return false,
            };
            let fits = match remaining {
                Some(left) => left >= n,
                None => true,
            };
            if fits {
                self.start_chunk(n);
                return self.alloc_fast(size, align);
            }
        }
    }

    fn try_alloc_layout(&self, size: usize, align: usize) -> Result<(), AllocErr> {
        if self.alloc_fast(size, align) || self.alloc_slow(size, align) { Ok(()) } else { Err(AllocErr) }
    }

    pub fn alloc<T>(&self, val: T) -> &mut T {
        self.alloc_with(|| val)
    }

    pub fn try_alloc<T>(&self, val: T) -> Result<&mut T, AllocErr> {
        self.try_alloc_with(|| val)
    }

    pub fn alloc_with<F: FnOnce() -> T, T>(&self, f: F) -> &mut T {
        if self.try_alloc_layout(core::mem::size_of::<T>(), core::mem::align_of::<T>()).is_err() {
            oom();
        }
        Box::leak(Box::new(f()))
    }

    /// Room first, then `f`: a failed allocation never calls `f`.
    pub fn try_alloc_with<F: FnOnce() -> T, T>(&self, f: F) -> Result<&mut T, AllocErr> {
        self.try_alloc_layout(core::mem::size_of::<T>(), core::mem::align_of::<T>())?;
        Ok(Box::leak(Box::new(f())))
    }

    pub fn alloc_slice_copy<T: Copy>(&self, src: &[T]) -> &mut [T] {
        if self.try_alloc_layout(core::mem::size_of::<T>() * src.len(), core::mem::align_of::<T>()).is_err() {
            oom();
        }
        Box::leak(src.to_vec().into_boxed_slice())
    }

    pub fn try_alloc_slice_copy<T: Copy>(&self, src: &[T]) -> Result<&mut [T], AllocErr> {
        self.try_alloc_layout(core::mem::size_of::<T>() * src.len(), core::mem::align_of::<T>())?;
        Ok(Box::leak(src.to_vec().into_boxed_slice()))
    }

    /// `len` items from `f(0)`, `f(1)`, ... in order.
    pub fn alloc_slice_fill_with<T, F: FnMut(usize) -> T>(&self, len: usize, f: F) -> &mut [T] {
        match self.try_fill_with(len, f) {
            Ok(s) => s,
            Err(_) => oom(),
        }
    }

    /// `alloc_slice_fill_with` that answers `AllocErr` instead of
    /// panicking when the room (or the layout) is not there; `f` is
    /// not called then.
    pub fn try_alloc_slice_fill_with<T, F: FnMut(usize) -> T>(&self, len: usize, f: F) -> Result<&mut [T], AllocErr> {
        self.try_fill_with(len, f)
    }

    /// Both fills: `Layout::array::<T>(len)`, the room, then `f(0)`,
    /// `f(1)`, ... in order. A panic in `f` leaks the items made so
    /// far (upstream writes them into the chunk) and gives the room
    /// back through `Rewind`.
    fn try_fill_with<T, F: FnMut(usize) -> T>(&self, len: usize, mut f: F) -> Result<&mut [T], AllocErr> {
        let bytes = match core::mem::size_of::<T>().checked_mul(len) {
            Some(b) if b <= isize::MAX as usize => b,
            _ => return Err(AllocErr),
        };
        let (chunk, ptr) = (self.allocated.get(), self.ptr.get());
        self.try_alloc_layout(bytes, core::mem::align_of::<T>())?;
        let mut guard = Rewind { bump: self, chunk, ptr, after_chunk: self.allocated.get(), after: self.ptr.get(), active: true };
        // The Vec is made in place and its buffer leaked as the slice (`Vec::leak`, no shrink
        // call): its 24-byte header is never copied whole right after a runtime call wrote
        // its fields, which stalls the copy's 16-byte load (most of this fn's cycles).
        let mut items = core::mem::ManuallyDrop::new(Vec::new());
        items.reserve_exact(len);
        let mut i = 0;
        while i < len {
            items.push(f(i));
            i += 1;
        }
        guard.active = false;
        Ok(core::mem::ManuallyDrop::into_inner(items).leak())
    }
}

/// Upstream's `RewindGuard` for a slice fill: when `f` panics and the
/// slice is still the last allocation, the bump position goes back to
/// where it was (same chunk), or to the start of the chunk made for it.
struct Rewind<'a> {
    bump: &'a Bump,
    /// The chunk (its `allocated` total) and position before the slice.
    chunk: usize,
    ptr: usize,
    /// The same after the slice's room was taken.
    after_chunk: usize,
    after: usize,
    active: bool,
}

impl Drop for Rewind<'_> {
    fn drop(&mut self) {
        let b = self.bump;
        // `is_last_allocation`: the bump position is still the slice's.
        if !self.active || b.allocated.get() != self.after_chunk || b.ptr.get() != self.after {
            return;
        }
        if self.after_chunk == self.chunk {
            b.ptr.set(self.ptr);
        } else {
            b.ptr.set(b.chunk_size.get());
        }
    }
}

impl Default for Bump {
    fn default() -> Bump {
        Bump::new()
    }
}
