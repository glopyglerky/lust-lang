// expect: L0733
// rustc E0521: a closure parameter `&dyn T` has the caller's object lifetime.
trait T { fn read(&self) -> u32; }
struct S { n: u32 }
impl T for S { fn read(&self) -> u32 { self.n } }
impl dyn T { fn more(&self) -> u32 { self.read() + 1 } }
static S0: S = S { n: 4 };
fn main() { let c = |d: &dyn T| d.more(); println!("{}", c(&S0)); }
