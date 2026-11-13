// expect: L0733
// rustc: lifetime may not live long enough (object lifetime inside a tuple).
trait T { fn read(&self) -> u32; }
struct S { n: u32 }
impl T for S { fn read(&self) -> u32 { self.n } }
fn bad(x: (&dyn T, u32)) -> (&(dyn T + 'static), u32) { x }
fn main() {}
