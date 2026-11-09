// expect: L0733
// rustc: lifetime may not live long enough. `v` reaches the return type under `&mut`, so its object lifetime is the caller's, but `more_mut` needs 'static.
trait T { fn read(&self) -> u32; }
struct S { n: u32 }
impl T for S { fn read(&self) -> u32 { self.n } }
impl dyn T { fn more_mut(&mut self) -> u32 { self.read() + 2 } }
fn bad(x: &mut S) -> Vec<&mut dyn T> { let d: &mut dyn T = x; d.more_mut(); let v = vec![d]; v }
fn main() {}
