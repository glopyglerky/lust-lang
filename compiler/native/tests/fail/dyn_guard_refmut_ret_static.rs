// expect: L0733
// rustc: lifetime may not live long enough. A `RefMut<'b, dyn P>` object may shorten to
// `'b` at a coercion, never lengthen to `'static`.
use std::cell::RefMut;
trait P { fn id(&self) -> u32; }
struct A(u32);
impl P for A { fn id(&self) -> u32 { self.0 } }
fn g<'b>(r: RefMut<'b, dyn P>) -> RefMut<'b, dyn P + 'static> {
    r
}
fn main() {}
