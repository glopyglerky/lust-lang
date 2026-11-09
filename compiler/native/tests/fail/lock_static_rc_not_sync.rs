// expect: L0612
use std::sync::Mutex;
use std::rc::Rc;
static M: Mutex<Option<Rc<i32>>> = Mutex::new(None);
fn main() { println!("{}", M.lock().unwrap().is_none()); }
