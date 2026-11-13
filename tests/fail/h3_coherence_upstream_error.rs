// expect: L0615
// rustc E0119: std may add `impl Error for i32` in a later release, so the
// blanket impl over T: Error may overlap the impl for i32.
trait Tr { fn f(&self) -> i32; }
impl<T: std::error::Error> Tr for T { fn f(&self) -> i32 { 1 } }
impl Tr for i32 { fn f(&self) -> i32 { 2 } }
fn main() { println!("{}", 5i32.f()); }
