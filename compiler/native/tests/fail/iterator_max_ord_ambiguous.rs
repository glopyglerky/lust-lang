// expect: L0104
// rustc E0034: an iterator that is also `Ord` has two by-value `max`
// methods (`Iterator::max` and `Ord::max`).
#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct I(u8);
impl Iterator for I {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        None
    }
}
fn main() {
    let x = I(1);
    println!("{:?}", x.max(I(2)).0);
}
