// expect: L0110
// rustc E0308: the trait's `&Self` is `&(dyn K + '_)`; the impl asks for more.
trait K { fn v(&self) -> u8; }
impl PartialEq for dyn K + '_ { fn eq(&self, o: &(dyn K + 'static)) -> bool { self.v() == o.v() } }
fn main() {}
