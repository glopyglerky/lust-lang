// str::get with every SliceIndex<str> range form (TRIAGE R26: the
// codegen tools' JSON parser reads `text.get(pos..pos + 4)`): None when
// a bound is out of range, reversed, or not on a char boundary.
use std::ops::Bound;
fn hex4(text: &str, pos: usize) -> Option<u32> {
    let digits = text.get(pos..pos + 4);
    digits.filter(|d| d.bytes().all(|b| b.is_ascii_hexdigit())).map(|d| d.len() as u32)
}
fn main() {
    let s = "hello wörld";
    println!("{:?} {:?} {:?} {:?} {:?}", s.get(0..5), s.get(6..), s.get(..20), s.get(1..=3), s.get(..));
    println!("{:?} {:?} {:?} {:?}", s.get(7..8), s.get(7..9), s.get(8..), s.get(5..3));
    let owned = String::from("abcdef");
    println!("{:?} {:?}", owned.get(2..4), owned.get(..=usize::MAX));
    let r = 1..3;
    println!("{:?} {:?}", s.get(r), s.get((Bound::Excluded(0), Bound::Included(2))));
    println!("{:?} {:?}", hex4("x00e9y", 1), hex4("x00", 1));
    let e = "";
    println!("{:?} {:?}", e.get(0..0), e.get(0..1));
}
