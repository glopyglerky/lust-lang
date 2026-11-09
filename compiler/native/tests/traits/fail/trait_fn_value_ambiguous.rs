// rustc-error: E0034
// lust-expect: L0720
// ARCH 2.10: two traits in scope give S a fn named mk; the plain path
// S::mk is ambiguous (a qualified path picks one).
trait A { fn mk() -> u8; }
trait B { fn mk() -> u8; }
struct S;
impl A for S { fn mk() -> u8 { 1 } }
impl B for S { fn mk() -> u8 { 2 } }
fn main() { let f = S::mk; println!("{}", f()); }
