// expect: L0110
// rustc: `impl` item signature doesn't match: `&dyn K` is `&'s (dyn K + 's)`, not `+ 'a`.
trait K { fn v(&self) -> u8; }
struct X(u8);
impl K for X { fn v(&self) -> u8 { self.0 } }
impl<'a> std::borrow::Borrow<dyn K + 'a> for X { fn borrow(&self) -> &dyn K { self } }
fn main() {}
