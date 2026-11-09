// expect: L0104
// rustc E0599: `Chain` is not an `ExactSizeIterator`, even over
// exact iterators, so it has no `len` method.
fn main() {
    let v = [1, 2];
    println!("{}", v.iter().chain(v.iter()).len());
}
