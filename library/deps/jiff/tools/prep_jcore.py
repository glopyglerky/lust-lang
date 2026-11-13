#!/usr/bin/env python3
"""Copy jiff-core 0.1.1's src into the jiff adapter as module `jcore`.

Usage: prep_jcore.py <jiff-core-0.1.1/src> library/deps/jiff/src/jcore
Mechanical edits:
- lib.rs becomes mod.rs without its inner attributes and extern crates;
- `crate::` paths gain `jcore::`; `alloc::` paths become `std::`.
Hand edits are in EDITS; each leaves a `lust:` comment in the copy.
"""
import os, re, sys, shutil

src, dst = sys.argv[1], sys.argv[2]
if os.path.exists(dst):
    shutil.rmtree(dst)
for root, _, files in os.walk(src):
    for f in files:
        if not f.endswith('.rs'):
            continue
        p = os.path.join(root, f)
        rel = os.path.relpath(p, src)
        if rel == 'lib.rs':
            rel = 'mod.rs'
        text = open(p).read()
        text = re.sub(r'(?<![\w$])crate::', 'crate::jcore::', text)
        text = re.sub(r'(?<![\w:])alloc::', 'std::', text)
        if rel == 'mod.rs':
            text = re.sub(r'^#!\[.*\]\n', '', text, flags=re.M)
            text = re.sub(r'^#\[cfg\(any\(test, feature = "(std|alloc)"\)\)\]\nextern crate \w+;\n\n?', '', text, flags=re.M)
        out = os.path.join(dst, rel)
        os.makedirs(os.path.dirname(out), exist_ok=True)
        open(out, 'w').write(text)

AS_STR_OLD = """        // SAFETY: Firstly, the unchecked UTF-8 conversion is correct because
        // the constructor and all mutators only accept `&str`. All mutators
        // (just `push_str` at time of writing) only ever concatenates the
        // given string to what is already there. Any UTF-8 string concatenated
        // with any other UTF-8 string produces a UTF-8 string.
        //
        // Secondly, `self.bytes` is always valid and initialized. And
        // `self.len` is managed as the length of bytes that make up the
        // string.
        unsafe {
            core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                self.bytes.as_ptr(),
                self.len as usize,
            ))
        }
"""
AS_STR_NEW = """        // lust: checked UTF-8 in place of upstream's `unsafe`
        // `from_utf8_unchecked(from_raw_parts(..))`, and not a `const fn`
        // (no const caller uses it). The bytes are always UTF-8 (see
        // upstream's SAFETY note), so this never panics.
        match core::str::from_utf8(&self.bytes[..self.len as usize]) {
            Ok(s) => s,
            Err(_) => panic!("ArrayStr holds UTF-8"),
        }
"""
WRITE_IMPL = """impl<const N: usize> core::fmt::Write for ArrayStr<N> {
    #[inline]
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        if self.push_str(s) {
            Ok(())
        } else {
            Err(core::fmt::Error)
        }
    }
}
"""
DIV_CEIL_OLD = """                // MSRV(1.73): Just use `n.div_ceil` instead.
                const fn div_ceil(lhs: u64, rhs: u64) -> u64 {
                    let d = lhs / rhs;
                    let r = lhs % rhs;
                    if r > 0 {
                        d + 1
                    } else {
                        d
                    }
                }
"""
DIV_CEIL_NEW = """
// lust: upstream's `div_ceil` from the const block in `Date::weekday`,
// moved here (lust's const evaluator does not take items in a block).
// MSRV(1.73): Just use `n.div_ceil` instead.
const fn weekday_div_ceil(lhs: u64, rhs: u64) -> u64 {
    let d = lhs / rhs;
    let r = lhs % rhs;
    if r > 0 {
        d + 1
    } else {
        d
    }
}
"""

RAW_OLD = """#[derive(Eq, PartialEq)]
pub struct RawBoundsError<B>(core::marker::PhantomData<B>);
"""
RAW_NEW = """// lust: derived Clone and Copy in place of upstream's written
// `impl<B> Copy` and `impl<B> Clone` (lust has no written Copy impls).
// Every `B` here is a boundary type, which is Copy, so both forms hold
// for the same types; `clone` returns the same unit value.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct RawBoundsError<B>(core::marker::PhantomData<B>);
"""
RAW_IMPLS_OLD = """impl<B> Copy for RawBoundsError<B> {}

impl<B> Clone for RawBoundsError<B> {
    #[inline]
    fn clone(&self) -> RawBoundsError<B> {
        RawBoundsError::new()
    }
}

"""

SIGN_F64_OLD = """/// This considers `NaN` values as having a zero sign.
///
/// In general, Jiff having a `NaN` value in memory is a bug.
impl From<f64> for Sign {
"""
HEADER = """// jiff-core 0.1.1's source (crates.io, MIT OR Unlicense), the crate jiff
// 0.2.37 builds on, copied for the lust jiff adapter. Upstream's text
// except for these edits (each is marked `lust:` where it is made):
// - lib.rs is this file, without its crate attributes and `extern crate`
//   lines; `crate::` paths are `crate::jcore::`, `alloc::` paths `std::`.
// - util/mod.rs: `ArrayStr::as_str` uses a checked `from_utf8` in place
//   of `from_utf8_unchecked` (no `unsafe`); it and `SmallStr::as_str` are
//   not `const fn`; `ArrayStr::<N>` / `SmallStrKind::<N>` are written out
//   in three constructors; `impl fmt::Write for ArrayStr` is cut (tests
//   only).
// - bounds.rs: the boundary types and `RawBoundsError` derive Clone and
//   Copy (upstream writes `impl<B> Copy` and `impl<B> Clone`); one
//   `<Self::Primitive as Primitive>::from_i64`; `Primitive`'s impls call
//   `<$intty>::try_from` and the method forms of `checked_add` and
//   `checked_mul`; `impl From<f64> for Sign` is cut (`#[cfg(any())]`).
// - civil/weekday.rs: the two `FusedIterator` impls are cut.
// - civil/date.rs: the `div_ceil` helper of `Date::weekday` is a
//   module-level fn (`weekday_div_ceil`).
// - civil/time.rs: `buf[end - 1]` for `buf[usize::from(end) - 1]`.
// - tz/posix.rs: `loop` for the two `for _ in 0..` loops.
// - tz/tzif/mod.rs: `MaybeNamedTimeZone::name` spells out `as_deref`.
// - tz/tzif/parser.rs: `lust_split_at(x, n)` (std's `split_at`, at the
//   end of that file) for `x.split_at(n)`; `while let Some(chunk) =
//   it.next()` for `for chunk in &mut it`; `(1..=bytes.len()).zip(..)`
//   for `(1..).zip(..)`.
// None of the edits changes a result or a panic. Regenerate with
// library/deps/jiff/tools/prep_jcore.py.

"""
EDITS = [
    ('util/mod.rs', AS_STR_OLD, AS_STR_NEW),
    ('util/mod.rs', WRITE_IMPL,
     '// lust: `impl fmt::Write for ArrayStr` cut (lust std has no fmt::Write\n'
     '// trait for user types; only upstream\'s tests use it).\n'),
    ('civil/weekday.rs', 'impl core::iter::FusedIterator for WeekdaysForward {}\n',
     '// lust: `impl FusedIterator for WeekdaysForward` cut (lust std has no\n'
     '// FusedIterator; the adapter never iterates weekdays).\n'),
    ('civil/weekday.rs', 'impl core::iter::FusedIterator for WeekdaysReverse {}\n',
     '// lust: `impl FusedIterator for WeekdaysReverse` cut (as above).\n'),
    ('civil/date.rs', DIV_CEIL_OLD,
     '                // lust: `div_ceil` moved out of this block\n'
     '                // (`weekday_div_ceil` at the end of this file).\n'),
    ('civil/date.rs', '                let n = div_ceil(1u64 << 32, 7) as u32;',
     '                let n = weekday_div_ceil(1u64 << 32, 7) as u32;'),
    ('bounds.rs', '            #[derive(Eq, PartialEq)]\n            pub struct $name(());',
     '            // lust: Clone and Copy added, so the derives on\n'
     '            // `RawBoundsError<$name>` (below) hold.\n'
     '            #[derive(Clone, Copy, Eq, PartialEq)]\n            pub struct $name(());'),
    ('bounds.rs', RAW_OLD, RAW_NEW),
    ('bounds.rs', RAW_IMPLS_OLD, ''),
    ('bounds.rs', '        Ok(Self::Primitive::from_i64(n))\n',
     '        // lust: the qualified form of `Self::Primitive::from_i64(n)`\n'
     '        // (same item: `from_i64` is `Primitive`\'s only such fn).\n'
     '        Ok(<Self::Primitive as Primitive>::from_i64(n))\n'),
    ('util/mod.rs', '    pub const fn as_str(&self) -> &str {',
     '    // lust: not a `const fn` (see `ArrayStr::as_str`).\n    pub fn as_str(&self) -> &str {', 2),
    ('util/mod.rs', '        let Some(astr) = ArrayStr::new(s) else { return None };',
     '        // lust: `ArrayStr::<N>` written out (lust does not infer a const\n'
     '        // argument from a later use).\n'
     '        let Some(astr) = ArrayStr::<ARRAY_CAPACITY_MAX>::new(s) else { return None };'),
    ('util/mod.rs', '        let Some(astr) = ArrayStr::new(s) else { panic!("string too big") };',
     '        // lust: `ArrayStr::<N>` written out (as in `try_array`).\n'
     '        let Some(astr) = ArrayStr::<ARRAY_CAPACITY_MAX>::new(s) else { panic!("string too big") };'),
    ('util/mod.rs', '        let kind = SmallStrKind::Static(s);\n        SmallStr { kind }',
     '        // lust: `SmallStrKind::<N>` written out (as in `try_array`).\n'
     '        let kind = SmallStrKind::<ARRAY_CAPACITY_MAX>::Static(s);\n        SmallStr { kind }'),
    ('bounds.rs', '                        Self::try_from(n).unwrap()\n',
     '                        // lust: `<$intty>` for `Self` (lust does not\n'
     '                        // resolve `Self::f` in an impl for a primitive).\n'
     '                        <$intty>::try_from(n).unwrap()\n', 4),
    ('bounds.rs', '                    <$intty>::checked_add(self, n)\n',
     '                    // lust: method form (the inherent method wins in\n'
     '                    // both; lust rejects the path form, L0643).\n'
     '                    self.checked_add(n)\n'),
    ('bounds.rs', '                    <$intty>::checked_mul(self, n)\n',
     '                    // lust: method form (as for `checked_add`).\n'
     '                    self.checked_mul(n)\n'),
    ('bounds.rs', SIGN_F64_OLD,
     '// lust: upstream\'s `impl From<f64> for Sign` is cut (lust std has no\n'
     '// `f64::classify`; the adapter never makes a Sign from a float).\n'
     '#[cfg(any())]\n'
     'impl From<f64> for Sign {\n'),
    ('tz/posix.rs', '        for _ in 0.. {\n',
     '        // lust: `loop` for `for _ in 0..` (lust has no open-range loops;\n'
     '        // the body ends within REASONABLE_ABBREVIATION_MAX steps).\n'
     '        loop {\n', 2),
    ('tz/tzif/mod.rs', '        self.name.as_deref()\n',
     '        // lust: `as_deref` spelled out (lust std has no `Option::as_deref`\n'
     '        // through a user `Deref`; `SmallStr::deref` is `as_str`).\n'
     '        self.name.as_ref().map(|name| name.as_str())\n'),
    ('civil/time.rs', "        while end > 0 && buf[usize::from(end) - 1] == b'0' {\n",
     "        // lust: `end` for `usize::from(end)` (`end` is a usize; lust\n"
     "        // picks the From impl before the later use fixes the type).\n"
     "        while end > 0 && buf[end - 1] == b'0' {\n"),
]

# Regex edits (file, pattern, replacement, expected count), after EDITS.
REGEX_EDITS = [
    # lust std has no `[T]::split_at` (L0643 for a same-named helper):
    # every call goes to `lust_split_at` (appended to parser.rs), which is
    # std's split_at. Every site splits in bounds.
    ('tz/tzif/parser.rs', r'\b(\w+)\.split_at\(', r'lust_split_at(\1, ', 12),
    # lust has no `impl Iterator for &mut I`: `while let` calls `next`
    # on the same iterator, as the `for` loop over `&mut it` does.
    ('tz/tzif/parser.rs', r'for chunk in &mut it \{', r'while let Some(chunk) = it.next() { // lust: for `for chunk in &mut it`', 3),
    # lust has no `zip` on an open range: `1..=bytes.len()` yields the
    # same indexes for the `bytes.len()` items.
    ('tz/tzif/parser.rs', r'in \(1\.\.\)\.zip\(bytes\)', r'in (1..=bytes.len()).zip(bytes) /* lust: for `(1..)` */', 1),
]
LUST_SPLIT_AT = """
// lust: std's `<[u8]>::split_at`, which lust's std lacks (see the
// `lust_split_at` calls above): the same split and the same panic.
fn lust_split_at(bytes: &[u8], mid: usize) -> (&[u8], &[u8]) {
    if mid > bytes.len() {
        panic!("mid > len");
    }
    (&bytes[..mid], &bytes[mid..])
}
"""
APPEND = [('civil/date.rs', DIV_CEIL_NEW), ('tz/tzif/parser.rs', LUST_SPLIT_AT)]

for rel, old, new, *count in EDITS:
    p = os.path.join(dst, rel)
    s = open(p).read()
    assert s.count(old) == (count[0] if count else 1), (rel, old[:60])
    open(p, 'w').write(s.replace(old, new))
for rel, pat, rep, n in REGEX_EDITS:
    p = os.path.join(dst, rel)
    s = open(p).read()
    s, k = re.subn(pat, rep, s)
    assert k == n, (rel, pat, k)
    open(p, 'w').write(s)
with open(os.path.join(dst, 'mod.rs')) as f:
    body = f.read()
with open(os.path.join(dst, 'mod.rs'), 'w') as f:
    f.write(HEADER + body)
for rel, text in APPEND:
    with open(os.path.join(dst, rel), 'a') as f:
        f.write(text)
