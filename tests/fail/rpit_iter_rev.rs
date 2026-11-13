// expect: L0612
// rustc E0277: an `impl Iterator` value is not DoubleEndedIterator, even
// when its hidden type is.
fn evens() -> impl Iterator<Item = u32> {
    vec![2u32, 4].into_iter()
}
fn main() {
    let v: Vec<u32> = evens().rev().collect();
    println!("{:?}", v);
}
