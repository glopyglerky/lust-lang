// char ranges as values: contains, is_empty, bounds, patterns, matches!, for loops (R28).
use std::ops::{Bound, RangeBounds};
fn is_digit(ch: char) -> bool { ('0'..='9').contains(&ch) }
fn is_hex(ch: char) -> bool { ('0'..='9').contains(&ch) || ('A'..='F').contains(&ch) || ('a'..='f').contains(&ch) }
fn class(c: char) -> &'static str {
    match c { 'a'..='z' => "lower", 'A'..='Z' => "upper", '0'..='9' => "digit", _ => "other" }
}
fn first<R: RangeBounds<char>>(r: R) -> Option<char> {
    match r.start_bound() { Bound::Included(&c) => Some(c), Bound::Excluded(&c) => char::from_u32(c as u32 + 1), Bound::Unbounded => None }
}
fn main() {
    for c in ['0', '9', 'a', 'f', 'g', 'F', 'Z', '!', '\u{D7FF}', '\u{E000}'] {
        println!("{c:?} {} {} {}", is_digit(c), is_hex(c), class(c));
    }
    let r = 'b'..'e';
    println!("{} {} {} {} {}", r.start, r.end, r.contains(&'e'), r.contains(&'b'), r.is_empty());
    let r = 'b'..='e';
    println!("{} {} {} {}", r.start(), r.end(), r.contains(&'e'), r.is_empty());
    println!("{} {} {}", ('c'..).contains(&'z'), (..'c').contains(&'c'), (..='c').contains(&'c'));
    println!("{} {}", ('z'..'a').is_empty(), ('z'..='a').is_empty());
    println!("{:?} {:?} {:?}", first('q'..'z'), first('q'..), first(..'z'));
    println!("{}", matches!('Q', 'A'..='Z'));
    let mut s = String::new();
    for c in 'a'..='e' { s.push(c); }
    for c in ('v'..'z').rev() { s.push(c); }
    let lo = '\u{D7FD}';
    for c in lo..'\u{E002}' { s.push_str(&format!(" {:x}", c as u32)); }
    println!("{s}");
    let mut r = char::MAX..=char::MAX;
    println!("{:?} {:?} {} {}", r.next(), r.start(), r.contains(&char::MAX), r.is_empty());
}
