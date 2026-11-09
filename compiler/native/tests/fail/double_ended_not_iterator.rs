// expect: L0612
// rustc E0277: `DoubleEndedIterator` needs `Iterator`.
struct X;
impl DoubleEndedIterator for X {
    fn next_back(&mut self) -> Option<u32> {
        None
    }
}
fn main() {
    let _x = X;
}
