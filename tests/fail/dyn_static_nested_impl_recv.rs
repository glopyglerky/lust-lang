// expect: L0733
// rustc E0521: the inherent impl is for `H<&(dyn T + 'static)>`; `h` holds a shorter object.
trait T { fn read(&self) -> u32; }
struct S { n: u32 }
impl T for S { fn read(&self) -> u32 { self.n } }
struct H<X>(X);
impl H<&(dyn T + 'static)> { fn m(&self) -> u32 { self.0.read() } }
fn bad(h: H<&dyn T>) -> u32 { h.m() }
fn main() {}
