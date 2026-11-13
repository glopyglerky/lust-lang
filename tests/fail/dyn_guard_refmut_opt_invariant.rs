// expect: L0705
// rustc: lifetime may not live long enough. `RefMut` is invariant in its `T`; only the guard
// itself coerces, not one inside an `Option`.
use std::cell::RefMut;
trait P { fn id(&self) -> u32; }
struct A(u32);
impl P for A { fn id(&self) -> u32 { self.0 } }
fn g<'b>(r: Option<RefMut<'b, dyn P + 'static>>) -> Option<RefMut<'b, dyn P>> {
    r
}
fn main() {}
