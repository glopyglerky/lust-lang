// expect: L0615
// rustc E0119: Duration is a std type; std may add `impl Iterator for Duration`.
trait Tr { fn f(&self) -> i32; }
impl<T: Iterator> Tr for T { fn f(&self) -> i32 { 1 } }
impl Tr for std::time::Duration { fn f(&self) -> i32 { 2 } }
fn main() { println!("{}", std::time::Duration::from_secs(1).f()); }
