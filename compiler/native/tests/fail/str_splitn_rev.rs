// expect: L0612
// rustc E0277: SplitN is not a DoubleEndedIterator.
fn main() {
    println!("{:?}", "a,b".splitn(2, ',').rev().count());
}
