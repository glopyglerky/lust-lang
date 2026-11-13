// expect: L0705
// rustc: lifetime may not live long enough. Under `&mut` the object lifetimes must match: the write would put a short-lived object behind `'static`.
trait T { fn read(&self) -> u32; }
struct S { n: u32 }
impl T for S { fn read(&self) -> u32 { self.n } }
fn bad(x: &mut &(dyn T + 'static), y: &dyn T) { let z: &mut &dyn T = x; *z = y; }
fn main() {}
