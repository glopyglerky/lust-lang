// expect: L0612
// rustc E0277: `RangeInclusive<i32>` (the integer fallback) is not ExactSizeIterator.
fn main() {
    let x = (1..=3).map(|x| x * 2).len();
    println!("{x}");
}
