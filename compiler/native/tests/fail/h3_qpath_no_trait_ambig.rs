// expect: L0720
// rustc E0034: two traits in scope give `m` for i32.
trait A { fn m(&self) -> i32 { 1 } }
trait B { fn m(&self) -> i32 { 2 } }
impl A for i32 {}
impl B for i32 {}
fn main() { println!("{}", <i32>::m(&5)); }
