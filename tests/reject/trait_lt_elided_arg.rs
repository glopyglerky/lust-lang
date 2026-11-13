// expect: L0203
// Stated cut (L0203): an elided (`'_`) lifetime argument of a trait. Valid Rust.
trait Visit<'d> {
    fn n(&self) -> usize;
}
struct K;
impl Visit<'_> for K {
    fn n(&self) -> usize {
        1
    }
}
fn main() {
    println!("{}", K.n());
}
