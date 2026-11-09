// expect: L0612
// rustc E0277: `RangeInclusive<i32>` (the integer fallback) is not an
// `ExactSizeIterator` (its length can overflow `usize`); only the 8-bit
// and 16-bit ones are.
fn main() {
    let mut it = 2..=5;
    it.next();
    println!("{}", it.len());
}
