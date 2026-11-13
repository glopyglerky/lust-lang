// expect: L0110
// rustc E0207: `'a` is used only in the associated type's object lifetime.
trait K { fn v(&self) -> u8; }
struct X(u8);
impl K for X { fn v(&self) -> u8 { self.0 } }
impl<'a> std::ops::Deref for X { type Target = dyn K + 'a; fn deref(&self) -> &(dyn K + 'a) { self } }
fn main() {}
