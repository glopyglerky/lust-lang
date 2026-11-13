// expect: L0733
// rustc: lifetime may not live long enough. The user Adt arg holds a shorter object lifetime.
trait T { fn read(&self) -> u32; }
struct S { n: u32 }
impl T for S { fn read(&self) -> u32 { self.n } }
struct H<X> { x: X }
fn bad(x: H<&dyn T>) -> H<&(dyn T + 'static)> { x }
fn main() {}
