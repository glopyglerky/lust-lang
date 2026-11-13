// expect: L0705
// rustc E0521: the method's `X: 'static` bound with `X = &dyn T` from a
// parameter (checked in `lust build` too, see inherent_generic_bound_build).
trait T { fn read(&self) -> u32; }
struct Z;
impl Z { fn m<X: 'static>(&self, _x: X) -> u32 { 1 } }
fn g(d: &dyn T) -> u32 { Z.m(d) }
fn main() {}
