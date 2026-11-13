// expect: L0705
// rustc: lifetime may not live long enough. `X: 'static` with `X = &dyn T` from a parameter.
trait T { fn read(&self) -> u32; }
struct S { n: u32 }
impl T for S { fn read(&self) -> u32 { self.n } }
fn need<X: 'static>(_x: X) {}
fn bad(d: &dyn T) { need(d); }
fn main() {}
