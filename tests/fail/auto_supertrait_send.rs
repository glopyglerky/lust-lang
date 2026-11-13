// expect: L0612
// rustc E0277: the supertrait Send must hold for the impl's Self type Rc<u8>.
use std::rc::Rc;
trait Tr: Send { fn v(&self) -> u8 { 1 } }
impl Tr for Rc<u8> {}
fn main() { println!("x"); }
