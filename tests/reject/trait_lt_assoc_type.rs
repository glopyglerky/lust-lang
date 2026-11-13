// expect: L0203
// Stated cut (L0203): an associated type in a trait with lifetime params.
// Valid Rust.
trait Source<'a> {
    type Item;
    fn first(&self) -> Option<&'a u8>;
}
struct S;
impl<'a> Source<'a> for S {
    type Item = u8;
    fn first(&self) -> Option<&'a u8> {
        None
    }
}
fn main() {
    println!("{:?}", S.first());
}
