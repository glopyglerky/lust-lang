// expect: L0733
// rustc: lifetime may not live long enough (object lifetime behind `&&`).
trait T { fn read(&self) -> u32; }
struct S { n: u32 }
impl T for S { fn read(&self) -> u32 { self.n } }
fn bad(d: &dyn T) -> u32 { let x: &&dyn T = &d; let y: &&(dyn T + 'static) = x; y.read() }
fn main() {}
