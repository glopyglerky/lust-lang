// expect: L0702
// ARCH 2.8: associated type bindings in dyn types are valid Rust; lust has
// no vtable form for them yet.
trait Source {
    type Item;
    fn get(&self) -> Self::Item;
}
struct S;
impl Source for S {
    type Item = u8;
    fn get(&self) -> u8 {
        4
    }
}
fn main() {
    let s = S;
    let d: &dyn Source<Item = u8> = &s;
    println!("{}", d.get());
}
