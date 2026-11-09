// expect: L0733
// rustc: lifetime may not live long enough (object lifetime inside `Option`).
trait T { fn read(&self) -> u32; }
struct S { n: u32 }
impl T for S { fn read(&self) -> u32 { self.n } }
fn bad(x: Option<&dyn T>) -> Option<&(dyn T + 'static)> { x }
fn main() {}
