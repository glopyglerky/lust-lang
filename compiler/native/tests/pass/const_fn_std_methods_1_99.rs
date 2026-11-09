// Std methods that are const fns in rustc 1.99 are callable in const fns:
// integer div_ceil/isqrt/abs_diff/midpoint and the mixed and strict
// families, char and u8 ASCII methods and encode_utf8, Cell::get and
// replace, slice and str ASCII queries, and RangeInclusive bounds. The
// integer and char ones also run in const items (the rest are outside
// the const evaluator for now).
use std::cell::Cell;

const fn ints(x: u32, y: i32) -> u64 {
    let a = x.div_ceil(3) as u64;
    let b = x.next_multiple_of(8) as u64;
    let c = x.isqrt() as u64;
    let d = x.abs_diff(1000) as u64;
    let e = u32::midpoint(x, 9) as u64;
    let f = match x.checked_add_signed(-5) {
        Some(v) => v as u64,
        None => 1,
    };
    let g = y.unsigned_abs() as u64;
    let h = x.wrapping_add_signed(y) as u64;
    let i = x.strict_add(1) as u64;
    let j = x.is_multiple_of(4) as u64;
    a + b + c + d + e + f + g + h + i + j
}

const fn chars(c: char) -> (bool, char) {
    (c.is_ascii_alphabetic() && !c.is_ascii_whitespace(), c.to_ascii_uppercase())
}

const fn upper(c: char) -> char {
    let mut m = c;
    m.make_ascii_uppercase();
    m
}

const fn utf8_len(c: char) -> usize {
    let mut b = [0u8; 4];
    c.encode_utf8(&mut b).len()
}

const fn cells() -> u8 {
    let c = Cell::new(5u8);
    let old = c.replace(6);
    old + c.get()
}

const fn text(s: &[u8], t: &str) -> bool {
    s.is_ascii() && t.eq_ignore_ascii_case("ABC")
}

const fn bounds() -> i32 {
    let r = 3..=9;
    *r.start() * *r.end()
}

const I: u64 = ints(17, -40);
const C: (bool, char) = chars('q');
const E: (bool, char) = chars('é');

fn main() {
    println!("{I} {C:?} {E:?}");
    println!("{} {:?} {}", ints(4000000000, i32::MIN), chars(' '), utf8_len('é'));
    println!("{} {} {} {} {}", cells(), text(b"xyz", "abc"), text(b"\xff", "abd"), bounds(), upper('k'));
}
