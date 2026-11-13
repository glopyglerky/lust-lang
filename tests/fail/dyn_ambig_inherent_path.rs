// expect: L0720
// ARCH 2.8, rustc E0034: `<dyn Tr>::m` sees the `impl dyn Tr` method and the object candidate `Tr::m`.
trait Tr { fn m(&self) -> u8; }
impl dyn Tr { fn m(&self) -> u8 { 100 } }
struct A;
impl Tr for A { fn m(&self) -> u8 { 1 } }
fn main() { let d: &dyn Tr = &A; println!("{}", <dyn Tr>::m(d)); }
