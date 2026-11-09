// expect: L0104
// rustc E0599: `Filter` is not an ExactSizeIterator, so it has no `len`.
fn main() {
    let v = vec![1, 2, 3];
    println!("{}", v.iter().filter(|x| **x > 0).len());
}
