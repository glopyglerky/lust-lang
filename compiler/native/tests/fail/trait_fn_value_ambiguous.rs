// expect: L0720
// rustc E0034: two traits in scope give S a fn named mk.
trait A { fn mk() -> u8; }
trait B { fn mk() -> u8; }
struct S;
impl A for S { fn mk() -> u8 { 1 } }
impl B for S { fn mk() -> u8 { 2 } }
fn main() { let f = S::mk; println!("{}", f()); }
