// expect: L0612
// rustc E0599: RangeFrom is not a DoubleEndedIterator.
fn main() {
    let mut r = 1u8..;
    println!("{:?}", r.next_back());
}
