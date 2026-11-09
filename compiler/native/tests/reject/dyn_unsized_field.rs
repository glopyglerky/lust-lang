// expect: L0110
// ARCH 2.7: a struct whose last field is `dyn Tr` is a valid unsized type in
// Rust; lust has no unsized structs yet and rejects the field.
trait Tr { fn m(&self) -> u8; }
struct S { a: u8, d: dyn Tr }
fn main() {}
