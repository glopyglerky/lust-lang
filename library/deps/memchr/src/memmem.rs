// Substring search (`memchr::memmem`): the free fns, the forward and
// reverse finders, and their match iterators. Matches are the leftmost
// (or rightmost) occurrence, as upstream returns. An empty needle
// matches at every position, including the haystack's end. Upstream
// writes `N: 'n` on `find_iter` and `rfind_iter`; `&'n N` implies it.

/// The first index where `needle` occurs in `haystack`.
pub fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    find_in(haystack, needle)
}

/// The last index where `needle` occurs in `haystack`.
pub fn rfind(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    rfind_in(haystack, needle)
}

/// An iterator over the non-overlapping matches of `needle`, front to back.
pub fn find_iter<'h, 'n, N: ?Sized + AsRef<[u8]>>(haystack: &'h [u8], needle: &'n N) -> FindIter<'h, 'n> {
    FindIter::new(haystack, Finder::new(needle))
}

/// An iterator over the matches of `needle`, back to front.
pub fn rfind_iter<'h, 'n, N: ?Sized + AsRef<[u8]>>(haystack: &'h [u8], needle: &'n N) -> FindRevIter<'h, 'n> {
    FindRevIter::new(haystack, FinderRev::new(needle))
}

/// The leftmost match, through the memmem intrinsic.
fn find_in(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    crate::memchr::found(core::intrinsics::memmem(haystack, needle))
}

fn rfind_in(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if haystack.len() < needle.len() {
        return None;
    }
    let mut i = haystack.len() - needle.len() + 1;
    while i > 0 {
        i -= 1;
        if starts_at(haystack, needle, i) {
            return Some(i);
        }
    }
    None
}

/// True when `needle` occurs in `haystack` at index `at` (in range).
fn starts_at(haystack: &[u8], needle: &[u8], at: usize) -> bool {
    let mut j = 0;
    while j < needle.len() {
        if haystack[at + j] != needle[j] {
            return false;
        }
        j += 1;
    }
    true
}

/// A needle the finder borrows, or owns after `into_owned`.
#[derive(Clone)]
enum CowBytes<'a> {
    Borrowed(&'a [u8]),
    Owned(Vec<u8>),
}

impl<'a> CowBytes<'a> {
    fn as_slice(&self) -> &[u8] {
        match self {
            CowBytes::Borrowed(s) => *s,
            CowBytes::Owned(v) => v.as_slice(),
        }
    }

    #[cfg(feature = "alloc")]
    fn into_owned(self) -> CowBytes<'static> {
        match self {
            CowBytes::Borrowed(s) => CowBytes::Owned(s.to_vec()),
            CowBytes::Owned(v) => CowBytes::Owned(v),
        }
    }
}

/// A forward substring searcher for one needle.
#[derive(Clone)]
pub struct Finder<'n> {
    needle: CowBytes<'n>,
}

impl<'n> Finder<'n> {
    pub fn new<B: ?Sized + AsRef<[u8]>>(needle: &'n B) -> Finder<'n> {
        Finder { needle: CowBytes::Borrowed(needle.as_ref()) }
    }

    pub fn find(&self, haystack: &[u8]) -> Option<usize> {
        find_in(haystack, self.needle.as_slice())
    }

    pub fn find_iter<'a, 'h>(&'a self, haystack: &'h [u8]) -> FindIter<'h, 'a> {
        FindIter::new(haystack, self.as_ref())
    }

    #[cfg(feature = "alloc")]
    pub fn into_owned(self) -> Finder<'static> {
        Finder { needle: self.needle.into_owned() }
    }

    pub fn as_ref(&self) -> Finder<'_> {
        Finder { needle: CowBytes::Borrowed(self.needle.as_slice()) }
    }

    pub fn needle(&self) -> &[u8] {
        self.needle.as_slice()
    }
}

/// A reverse substring searcher for one needle.
#[derive(Clone)]
pub struct FinderRev<'n> {
    needle: CowBytes<'n>,
}

impl<'n> FinderRev<'n> {
    pub fn new<B: ?Sized + AsRef<[u8]>>(needle: &'n B) -> FinderRev<'n> {
        FinderRev { needle: CowBytes::Borrowed(needle.as_ref()) }
    }

    pub fn rfind<B: AsRef<[u8]>>(&self, haystack: B) -> Option<usize> {
        rfind_in(haystack.as_ref(), self.needle.as_slice())
    }

    pub fn rfind_iter<'a, 'h>(&'a self, haystack: &'h [u8]) -> FindRevIter<'h, 'a> {
        FindRevIter::new(haystack, self.as_ref())
    }

    #[cfg(feature = "alloc")]
    pub fn into_owned(self) -> FinderRev<'static> {
        FinderRev { needle: self.needle.into_owned() }
    }

    pub fn as_ref(&self) -> FinderRev<'_> {
        FinderRev { needle: CowBytes::Borrowed(self.needle.as_slice()) }
    }

    pub fn needle(&self) -> &[u8] {
        self.needle.as_slice()
    }
}

/// The non-overlapping matches of one needle, front to back. After a
/// match the search resumes past it (one byte on for an empty needle).
#[derive(Clone)]
pub struct FindIter<'h, 'n> {
    haystack: &'h [u8],
    finder: Finder<'n>,
    pos: usize,
}

impl<'h, 'n> FindIter<'h, 'n> {
    fn new(haystack: &'h [u8], finder: Finder<'n>) -> FindIter<'h, 'n> {
        FindIter { haystack, finder, pos: 0 }
    }

    #[cfg(feature = "alloc")]
    pub fn into_owned(self) -> FindIter<'h, 'static> {
        FindIter { haystack: self.haystack, finder: self.finder.into_owned(), pos: self.pos }
    }
}

impl<'h, 'n> Iterator for FindIter<'h, 'n> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        if self.pos > self.haystack.len() {
            return None;
        }
        let needle = self.finder.needle();
        let idx = find_in(&self.haystack[self.pos..], needle)?;
        let pos = self.pos + idx;
        self.pos = pos + needle.len().max(1);
        Some(pos)
    }

    /// Upstream's bound: the most non-overlapping matches left, and an
    /// empty needle matches at every position, the end included.
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.haystack.len().checked_sub(self.pos) {
            None => (0, Some(0)),
            Some(haystack_len) => match self.finder.needle().len() {
                0 => (haystack_len.saturating_add(1), haystack_len.checked_add(1)),
                needle_len => (0, Some(haystack_len / needle_len)),
            },
        }
    }
}

/// The matches of one needle, back to front. A match at the current end
/// steps the end back one byte (an empty needle matches everywhere).
#[derive(Clone)]
pub struct FindRevIter<'h, 'n> {
    haystack: &'h [u8],
    finder: FinderRev<'n>,
    pos: Option<usize>,
}

impl<'h, 'n> FindRevIter<'h, 'n> {
    fn new(haystack: &'h [u8], finder: FinderRev<'n>) -> FindRevIter<'h, 'n> {
        FindRevIter { haystack, finder, pos: Some(haystack.len()) }
    }

    #[cfg(feature = "alloc")]
    pub fn into_owned(self) -> FindRevIter<'h, 'static> {
        FindRevIter { haystack: self.haystack, finder: self.finder.into_owned(), pos: self.pos }
    }
}

impl<'h, 'n> Iterator for FindRevIter<'h, 'n> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        let pos = self.pos?;
        let i = rfind_in(&self.haystack[..pos], self.finder.needle())?;
        if pos == i {
            self.pos = pos.checked_sub(1);
        } else {
            self.pos = Some(i);
        }
        Some(i)
    }
}
