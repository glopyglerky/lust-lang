// expect: L0733
// rustc: lifetime may not live long enough (object lifetime inside an array).
trait T { fn read(&self) -> u32; }
struct S { n: u32 }
impl T for S { fn read(&self) -> u32 { self.n } }
fn bad(x: [&dyn T; 1]) -> [&(dyn T + 'static); 1] { x }
fn main() {}
