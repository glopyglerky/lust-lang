// `use std::m::{self, ..}` binds the std module name, at module level
// and in a fn body.
use std::process::{self, ExitCode};
mod inner {
    use std::rc::{self, Rc};
    pub fn run() -> String {
        let a: rc::Rc<i32> = Rc::new(1);
        let w: rc::Weak<i32> = Rc::downgrade(&a);
        format!("{} {:?}", a, w.upgrade())
    }
}
fn main() -> ExitCode {
    use std::cell::{self, Cell};
    let c: cell::Cell<u8> = Cell::new(3);
    println!("{} {}", inner::run(), c.get());
    if c.get() == 0 {
        process::exit(1);
    }
    ExitCode::from(c.get())
}
