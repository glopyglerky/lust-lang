// expect: L0612
// rustc E0277: there is no `Probe` impl for the array type; the
// slice impl is only reached by a method call's autoderef step
// or by naming the slice impl in the qualified path.
trait Probe {
    fn total(&self) -> u32;
}
impl Probe for [u8] {
    fn total(&self) -> u32 {
        self.len() as u32
    }
}
fn main() {
    println!("{}", <[u8; 3] as Probe>::total(&[1u8, 2, 3]));
}
