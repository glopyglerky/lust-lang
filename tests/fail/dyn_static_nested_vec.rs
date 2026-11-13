// expect: L0733
// rustc: lifetime may not live long enough. The object lifetime inside `Vec` cannot grow to 'static.
trait T { fn read(&self) -> u32; }
struct S { n: u32 }
impl T for S { fn read(&self) -> u32 { self.n } }
fn bad(x: Vec<&dyn T>) -> Vec<&(dyn T + 'static)> { x }
fn main() {}
