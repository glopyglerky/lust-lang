// `core::char` overlay (docs/ARCH.md section 3.1; TRIAGE T8):
// `ToLowercase` and `ToUppercase`, the iterators behind
// `char::to_lowercase` and `char::to_uppercase`. The other `char`
// items stay compiler-known; `core::char::X` and `std::char::X`
// walk into this module only for the names it defines
// (`STD_MODULES` in src/stdmods.rs).
//
// Lust std source, adapted from rust-lang/rust 1.99
// `core/src/char/case.rs`. Std maps through its `CaseMappingIter`
// (one to three chars in an array); here the privileged
// `core::intrinsics::char_case` runs the same Unicode tables in
// the runtime (the ones `str::to_lowercase` uses) and packs the
// result: code points in the low word, their count in the high
// word. No final-sigma rule: that is `str::to_lowercase` only, as
// in std.

/// The state both case iterators carry: the mapped code points and
/// the front and back (exclusive) indexes still to yield.
#[derive(Clone)]
struct CaseState {
    chars: [u32; 3],
    front: u8,
    back: u8,
}

impl CaseState {
    /// A state holding the case mapping of `c`.
    fn new(c: char, upper: bool) -> CaseState {
        let packed = core::intrinsics::char_case(c as u32, upper);
        let lo = packed as u64;
        let chars = [(lo & 0x1F_FFFF) as u32, ((lo >> 21) & 0x1F_FFFF) as u32, ((lo >> 42) & 0x1F_FFFF) as u32];
        CaseState { chars, front: 0, back: (packed >> 64) as u8 }
    }

    /// The chars left written out (the `Display` of both iterators).
    fn display(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut i = self.front;
        while i < self.back {
            match char::from_u32(self.chars[i as usize]) {
                Some(c) => write!(f, "{}", c)?,
                None => {}
            }
            i += 1;
        }
        Ok(())
    }
}

/// An iterator that yields the lowercase mapping of a char
/// (`char::to_lowercase`).
#[derive(Clone)]
pub struct ToLowercase {
    state: CaseState,
}

/// An iterator that yields the uppercase mapping of a char
/// (`char::to_uppercase`).
#[derive(Clone)]
pub struct ToUppercase {
    state: CaseState,
}

impl Iterator for ToLowercase {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        if self.state.front >= self.state.back {
            return None;
        }
        let c = self.state.chars[self.state.front as usize];
        self.state.front += 1;
        char::from_u32(c)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = (self.state.back - self.state.front) as usize;
        (n, Some(n))
    }
}

impl Iterator for ToUppercase {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        if self.state.front >= self.state.back {
            return None;
        }
        let c = self.state.chars[self.state.front as usize];
        self.state.front += 1;
        char::from_u32(c)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = (self.state.back - self.state.front) as usize;
        (n, Some(n))
    }
}

impl DoubleEndedIterator for ToLowercase {
    fn next_back(&mut self) -> Option<char> {
        if self.state.front >= self.state.back {
            return None;
        }
        self.state.back -= 1;
        char::from_u32(self.state.chars[self.state.back as usize])
    }
}

impl DoubleEndedIterator for ToUppercase {
    fn next_back(&mut self) -> Option<char> {
        if self.state.front >= self.state.back {
            return None;
        }
        self.state.back -= 1;
        char::from_u32(self.state.chars[self.state.back as usize])
    }
}

impl ExactSizeIterator for ToLowercase {}
impl ExactSizeIterator for ToUppercase {}

/// Std's `CaseMappingIter(IntoIter([..]))`: the private iterator
/// both case iterators wrap, over a `char` array `IntoIter`.
impl core::fmt::Debug for CaseState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("CaseMappingIter").field(&CaseChars(self)).finish()
    }
}

/// The chars left, as std's array `IntoIter` Debug prints them:
/// `IntoIter([..])` of the remaining slice.
struct CaseChars<'a>(&'a CaseState);

impl core::fmt::Debug for CaseChars<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut chars = ['\0'; 3];
        let mut i = 0;
        while i < 3 {
            if let Some(c) = char::from_u32(self.0.chars[i]) {
                chars[i] = c;
            }
            i += 1;
        }
        f.debug_tuple("IntoIter").field(&&chars[self.0.front as usize..self.0.back as usize]).finish()
    }
}

impl core::fmt::Debug for ToLowercase {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("ToLowercase").field(&self.state).finish()
    }
}

impl core::fmt::Debug for ToUppercase {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("ToUppercase").field(&self.state).finish()
    }
}

impl core::fmt::Display for ToLowercase {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.state.display(f)
    }
}

impl core::fmt::Display for ToUppercase {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.state.display(f)
    }
}

impl char {
    /// The lowercase mapping of this char, as in std.
    pub fn to_lowercase(self) -> ToLowercase {
        ToLowercase { state: CaseState::new(self, false) }
    }

    /// The uppercase mapping of this char, as in std.
    pub fn to_uppercase(self) -> ToUppercase {
        ToUppercase { state: CaseState::new(self, true) }
    }
}

// `char::decode_utf16` (std `core/src/char/decode.rs`): UTF-16 code
// units to chars, an unpaired surrogate as an error. No Debug for the
// iterator (it would print the inner iterator's Debug) and no
// `Error` impl here: both reject.

/// An iterator that decodes UTF-16 encoded code points from an iterator
/// of `u16`s.
#[derive(Clone)]
pub struct DecodeUtf16<I>
where
    I: Iterator<Item = u16>,
{
    iter: I,
    buf: Option<u16>,
}

/// An error that can be returned when decoding UTF-16 code points.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DecodeUtf16Error {
    code: u16,
}

impl<I: Iterator<Item = u16>> Iterator for DecodeUtf16<I> {
    type Item = Result<char, DecodeUtf16Error>;

    fn next(&mut self) -> Option<Result<char, DecodeUtf16Error>> {
        let u = match self.buf.take() {
            Some(buf) => buf,
            None => self.iter.next()?,
        };

        // `from_u32` cannot fail on these values (std uses the unchecked
        // form); `unwrap_or` keeps this std source free of panic sites.
        if !(0xD800..=0xDFFF).contains(&u) {
            Some(Ok(char::from_u32(u as u32).unwrap_or('\0')))
        } else if u >= 0xDC00 {
            Some(Err(DecodeUtf16Error { code: u }))
        } else {
            let u2 = match self.iter.next() {
                Some(u2) => u2,
                None => return Some(Err(DecodeUtf16Error { code: u })),
            };
            if u2 < 0xDC00 || u2 > 0xDFFF {
                self.buf = Some(u2);
                return Some(Err(DecodeUtf16Error { code: u }));
            }
            let c = (((u & 0x3ff) as u32) << 10 | (u2 & 0x3ff) as u32) + 0x1_0000;
            Some(Ok(char::from_u32(c).unwrap_or('\0')))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (low, high) = self.iter.size_hint();
        let (low_buf, high_buf) = match self.buf {
            None => (0, 0),
            Some(u) if !(0xD800..=0xDFFF).contains(&u) => (1, 1),
            Some(_u) if high == Some(0) => (1, 1),
            Some(_u) => (0, 1),
        };
        let low = low.div_ceil(2) + low_buf;
        let high = high.and_then(|h| h.checked_add(high_buf));
        (low, high)
    }
}

impl DecodeUtf16Error {
    /// Returns the unpaired surrogate which caused this error.
    pub fn unpaired_surrogate(&self) -> u16 {
        self.code
    }
}

impl core::fmt::Display for DecodeUtf16Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "unpaired surrogate found: {:x}", self.code)
    }
}

impl char {
    /// Creates an iterator over the UTF-16 encoded code points in `iter`,
    /// returning unpaired surrogates as `Err`s.
    // Std takes any `IntoIterator<Item = u16>` and returns
    // `DecodeUtf16<I::IntoIter>`; lust cannot prove the struct's bound on
    // that projection yet, so only iterators are taken here (the same
    // type for them; other arguments reject).
    pub fn decode_utf16<I: Iterator<Item = u16>>(iter: I) -> DecodeUtf16<I> {
        DecodeUtf16 { iter, buf: None }
    }
}
