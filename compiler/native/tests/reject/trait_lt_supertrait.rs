// expect: L0203
// Stated cut (L0203): a lifetime argument in a trait's own bounds needs a
// region replay at every impl. Valid Rust.
trait Base<'a> {
    fn get(&self) -> Option<&'a u8>;
}
trait Derived<'a>: Base<'a> {
    fn twice(&self) -> usize {
        2
    }
}
struct S;
impl<'a> Base<'a> for S {
    fn get(&self) -> Option<&'a u8> {
        None
    }
}
impl<'a> Derived<'a> for S {}
fn main() {
    println!("{}", S.twice());
}
