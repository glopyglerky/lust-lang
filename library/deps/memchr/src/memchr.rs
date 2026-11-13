// Single-byte searches: one, two, or three needle bytes, forward and
// reverse, plus the iterators over all matches. One needle goes through
// the memchr and memrchr intrinsics (libc, src/runtime/deps.c), two and
// three through memchr3 and memrchr3 (the runtime's vector loops).

/// The first index of `needle` in `haystack`.
pub fn memchr(needle: u8, haystack: &[u8]) -> Option<usize> {
    found(core::intrinsics::memchr(needle as u32, haystack))
}

/// The last index of `needle` in `haystack`.
pub fn memrchr(needle: u8, haystack: &[u8]) -> Option<usize> {
    found(core::intrinsics::memrchr(needle as u32, haystack))
}

/// An intrinsic's index answer: usize::MAX means no match.
pub(crate) fn found(i: usize) -> Option<usize> {
    if i == usize::MAX { None } else { Some(i) }
}

/// The first index of either needle byte.
pub fn memchr2(needle1: u8, needle2: u8, haystack: &[u8]) -> Option<usize> {
    memchr3_in(needle1, needle2, needle2, haystack, 0, haystack.len())
}

/// The last index of either needle byte.
pub fn memrchr2(needle1: u8, needle2: u8, haystack: &[u8]) -> Option<usize> {
    memrchr3_in(needle1, needle2, needle2, haystack, 0, haystack.len())
}

/// The first index of any of the three needle bytes.
pub fn memchr3(needle1: u8, needle2: u8, needle3: u8, haystack: &[u8]) -> Option<usize> {
    memchr3_in(needle1, needle2, needle3, haystack, 0, haystack.len())
}

/// The last index of any of the three needle bytes.
pub fn memrchr3(needle1: u8, needle2: u8, needle3: u8, haystack: &[u8]) -> Option<usize> {
    memrchr3_in(needle1, needle2, needle3, haystack, 0, haystack.len())
}

/// An iterator over every index of `needle`, front to back.
pub fn memchr_iter<'h>(needle: u8, haystack: &'h [u8]) -> Memchr<'h> {
    Memchr::new(needle, haystack)
}

/// An iterator over every index of either needle byte.
pub fn memchr2_iter<'h>(needle1: u8, needle2: u8, haystack: &'h [u8]) -> Memchr2<'h> {
    Memchr2::new(needle1, needle2, haystack)
}

/// An iterator over every index of any of the three needle bytes.
pub fn memchr3_iter<'h>(needle1: u8, needle2: u8, needle3: u8, haystack: &'h [u8]) -> Memchr3<'h> {
    Memchr3::new(needle1, needle2, needle3, haystack)
}

/// The first index in `start..end` holding one of the needles. Callers
/// keep `start <= end <= haystack.len()`: one range check, then the
/// runtime's search of the sub-slice (the memchr3 intrinsic, 16 bytes a
/// step where it can, as upstream's vector searches).
fn memchr3_in(n1: u8, n2: u8, n3: u8, haystack: &[u8], start: usize, end: usize) -> Option<usize> {
    debug_assert!(start <= end && end <= haystack.len());
    let i = found(core::intrinsics::memchr3(n1 as u32, n2 as u32, n3 as u32, &haystack[start..end]))?;
    Some(start + i)
}

/// The last index in `start..end` holding one of the needles (as
/// `memchr3_in`, from the back).
fn memrchr3_in(n1: u8, n2: u8, n3: u8, haystack: &[u8], start: usize, end: usize) -> Option<usize> {
    debug_assert!(start <= end && end <= haystack.len());
    let i = found(core::intrinsics::memrchr3(n1 as u32, n2 as u32, n3 as u32, &haystack[start..end]))?;
    Some(start + i)
}

/// Every index of one needle byte. The unsearched range is `start..end`,
/// as upstream's pointer pair: `next` moves `start` past a match,
/// `next_back` moves `end` onto one, and `size_hint` is upstream's
/// `(0, Some(end - start))` (tests/adapters/crates/memchr/iter_hint).
#[derive(Clone)]
pub struct Memchr<'h> {
    needle1: u8,
    haystack: &'h [u8],
    start: usize,
    end: usize,
}

impl<'h> Memchr<'h> {
    pub fn new(needle1: u8, haystack: &'h [u8]) -> Memchr<'h> {
        Memchr { needle1, haystack, start: 0, end: haystack.len() }
    }
}

impl<'h> Iterator for Memchr<'h> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        let i = self.start + memchr(self.needle1, &self.haystack[self.start..self.end])?;
        self.start = i + 1;
        Some(i)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.end.saturating_sub(self.start)))
    }
}

impl<'h> DoubleEndedIterator for Memchr<'h> {
    fn next_back(&mut self) -> Option<usize> {
        let i = self.start + memrchr(self.needle1, &self.haystack[self.start..self.end])?;
        self.end = i;
        Some(i)
    }
}

/// Every index of either of two needle bytes. See `Memchr`.
#[derive(Clone)]
pub struct Memchr2<'h> {
    needle1: u8,
    needle2: u8,
    haystack: &'h [u8],
    start: usize,
    end: usize,
}

impl<'h> Memchr2<'h> {
    pub fn new(needle1: u8, needle2: u8, haystack: &'h [u8]) -> Memchr2<'h> {
        Memchr2 { needle1, needle2, haystack, start: 0, end: haystack.len() }
    }
}

impl<'h> Iterator for Memchr2<'h> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        let i = memchr3_in(self.needle1, self.needle2, self.needle2, self.haystack, self.start, self.end)?;
        self.start = i + 1;
        Some(i)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.end.saturating_sub(self.start)))
    }
}

impl<'h> DoubleEndedIterator for Memchr2<'h> {
    fn next_back(&mut self) -> Option<usize> {
        let i = memrchr3_in(self.needle1, self.needle2, self.needle2, self.haystack, self.start, self.end)?;
        self.end = i;
        Some(i)
    }
}

/// Every index of any of three needle bytes. See `Memchr`.
#[derive(Clone)]
pub struct Memchr3<'h> {
    needle1: u8,
    needle2: u8,
    needle3: u8,
    haystack: &'h [u8],
    start: usize,
    end: usize,
}

impl<'h> Memchr3<'h> {
    pub fn new(needle1: u8, needle2: u8, needle3: u8, haystack: &'h [u8]) -> Memchr3<'h> {
        Memchr3 { needle1, needle2, needle3, haystack, start: 0, end: haystack.len() }
    }
}

impl<'h> Iterator for Memchr3<'h> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        let i = memchr3_in(self.needle1, self.needle2, self.needle3, self.haystack, self.start, self.end)?;
        self.start = i + 1;
        Some(i)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.end.saturating_sub(self.start)))
    }
}

impl<'h> DoubleEndedIterator for Memchr3<'h> {
    fn next_back(&mut self) -> Option<usize> {
        let i = memrchr3_in(self.needle1, self.needle2, self.needle3, self.haystack, self.start, self.end)?;
        self.end = i;
        Some(i)
    }
}

