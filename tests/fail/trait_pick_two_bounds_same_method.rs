// expect: L0720
// Two where-clause traits of one parameter declare `f`: ambiguous (rustc E0034).
trait A { fn f(&self) -> u8; }
trait B { fn f(&self) -> u8; }
impl A for u8 { fn f(&self) -> u8 { 1 } }
impl B for u8 { fn f(&self) -> u8 { 2 } }
fn g<T: A + B>(t: &T) -> u8 { t.f() }
fn main() { println!("{}", g(&5u8)); }
