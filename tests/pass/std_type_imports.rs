// Non-prelude std types come into scope by an explicit import, a
// rename, a module import, or a glob, at module or fn-body level.
mod by_glob {
    use std::rc::*;
    pub fn run() -> String {
        let a = Rc::new(1);
        let w: Weak<i32> = Rc::downgrade(&a);
        let e = Weak::<u8>::new();
        format!("{:?} {:?}", w.upgrade(), e.upgrade())
    }
}
mod by_local_glob {
    pub fn run() -> String {
        use std::cell::*;
        let c = Cell::new(1);
        let r = RefCell::new(vec![1]);
        c.set(c.get() + 1);
        r.borrow_mut().push(2);
        format!("{} {:?}", c.get(), r.borrow())
    }
}
mod by_module {
    use std::rc;
    pub fn run() -> String {
        let a = rc::Rc::new(1);
        let w = rc::Rc::downgrade(&a);
        let e = rc::Weak::<u8>::new();
        format!("{:?} {:?} {}", w.upgrade(), e.upgrade(), rc::Rc::weak_count(&a))
    }
}
mod by_rename {
    use std::cell::RefCell as Cell;
    pub fn run() -> String {
        let c = Cell::new(3);
        *c.borrow_mut() += 1;
        format!("{}", c.borrow())
    }
}
mod nested_fn {
    pub fn run() -> i32 {
        use std::cell::Cell;
        fn f() -> Cell<i32> {
            Cell::new(1)
        }
        f().get()
    }
}
mod reexport {
    pub mod a {
        pub use std::cell::Cell;
    }
    use self::a::Cell;
    pub fn run() -> i32 {
        Cell::new(3).get()
    }
}
fn main() {
    println!("{}", by_glob::run());
    println!("{}", by_local_glob::run());
    println!("{}", by_module::run());
    println!("{}", by_rename::run());
    println!("{} {}", nested_fn::run(), reexport::run());
}
