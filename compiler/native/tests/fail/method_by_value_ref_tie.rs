// expect: L0720
// rustc E0034: a by-value method of &N and a &self method of N tie.
trait A { fn f(self) -> u8; }
trait B { fn f(&self) -> u8; }
struct N;
impl A for &N { fn f(self) -> u8 { 1 } }
impl B for N { fn f(&self) -> u8 { 2 } }
fn main() { let n = N; let r = &n; println!("{}", r.f()); }
