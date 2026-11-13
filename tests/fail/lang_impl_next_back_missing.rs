// expect: L0110
// rustc E0046: a `DoubleEndedIterator` impl must define `next_back`.
struct N(u32);
impl Iterator for N {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        None
    }
}
impl DoubleEndedIterator for N {}
fn main() {
    println!("{}", N(0).count());
}
