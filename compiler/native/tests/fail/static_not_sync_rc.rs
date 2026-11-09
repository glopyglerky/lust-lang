// expect: L0612
use std::rc::Rc;
struct Holder { r: Option<Rc<u8>> }
static H: Holder = Holder { r: None };
fn main() { println!("{}", H.r.is_none()); }
