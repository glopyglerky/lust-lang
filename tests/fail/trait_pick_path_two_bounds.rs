// expect: L0720
// `T::make()` with two bound traits that both declare `make` (rustc E0034).
trait A { fn make() -> u8; }
trait B { fn make() -> u8; }
impl A for u8 { fn make() -> u8 { 1 } }
impl B for u8 { fn make() -> u8 { 2 } }
fn g<T: A + B>() -> u8 { T::make() }
fn main() { println!("{}", g::<u8>()); }
