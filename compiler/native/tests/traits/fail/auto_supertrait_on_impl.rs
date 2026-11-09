// rustc-error: E0277
// lust-expect: L0612
// ARCH 2.7: an impl owes its trait's Send supertrait at collect time.
use std::rc::Rc;
trait Tr: Send { fn v(&self) -> u8 { 1 } }
impl Tr for Rc<u8> {}
fn main() { println!("x"); }
