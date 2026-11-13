// expect: L0640
// rustc E0277: Filter is not an ExactSizeIterator, so it has no rposition.
fn main() {
    let v = vec![1, 2, 3];
    println!("{:?}", v.iter().filter(|x| **x > 1).rposition(|x| *x == 2));
}
