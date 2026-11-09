// User types named like non-prelude std types: a user type shadows
// std's bare name, and a std path or import still means std's type.
struct Rc<T>(T);
impl Rc<i32> {
    fn new(v: i32) -> Rc<i32> {
        Rc(v + 10)
    }
}
struct Cell<T>(T);
impl Cell<i32> {
    fn new(v: i32) -> Cell<i32> {
        Cell(v + 10)
    }
    fn get(&self) -> i32 {
        self.0
    }
}
mod uses_std {
    use std::rc::{Rc, Weak};
    pub fn run() -> String {
        let a: Rc<i32> = Rc::new(1);
        let w: Weak<i32> = Rc::downgrade(&a);
        let c = std::cell::Cell::new(1);
        c.set(c.get() + 1);
        format!("{} {} {}", *a, w.upgrade().is_some(), c.get())
    }
}
mod own {
    pub struct Weak(pub i32);
    impl Weak {
        pub fn new() -> Weak {
            Weak(7)
        }
    }
}
fn main() {
    let r = Rc::new(1);
    let c = Cell::new(2);
    println!("{} {} {}", r.0, c.get(), own::Weak::new().0);
    println!("{}", uses_std::run());
}
