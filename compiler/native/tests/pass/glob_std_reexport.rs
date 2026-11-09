// A glob import brings the std items a module re-exports (`pub use std::...`),
// through nested globs, `{self, Item}` groups, renames, `use super::*`, and
// fn-body globs. A local item or explicit import of the name wins.
mod prelude {
    pub use std::cell::RefCell;
    pub use std::collections::{self, HashMap};
    pub use std::fmt::Write as _;
    pub use std::rc::Rc;
    pub use std::rc::Weak as RcWeak;
}

mod util {
    pub use crate::prelude::*;
    pub fn count(m: &HashMap<u8, u8>) -> usize {
        m.len()
    }
}

mod deep {
    use crate::util::*;
    pub fn make() -> Rc<RefCell<Vec<u8>>> {
        let r = Rc::new(RefCell::new(vec![1]));
        let w: RcWeak<RefCell<Vec<u8>>> = Rc::downgrade(&r);
        w.upgrade().unwrap().borrow_mut().push(2);
        r
    }
    pub fn set() -> collections::HashSet<u8> {
        let mut s = collections::HashSet::new();
        s.insert(4);
        s
    }
}

mod local_wins {
    use crate::prelude::*;
    // A local item beats the glob's `Rc`.
    pub struct Rc(pub u8);
    pub fn get() -> u8 {
        let m: HashMap<u8, u8> = HashMap::new();
        Rc(7).0 + m.len() as u8
    }
}

mod parent {
    use std::rc::Rc;
    pub fn shared() -> Rc<u8> {
        Rc::new(9)
    }
    pub mod child {
        // `use super::*` brings the parent's private std import.
        use super::*;
        pub fn again() -> Rc<u8> {
            let a = shared();
            Rc::clone(&a)
        }
    }
}

use util::*;

fn body_glob() -> u32 {
    use crate::prelude::*;
    let c = RefCell::new(5u32);
    *c.borrow_mut() += 1;
    let x = c.into_inner();
    x
}

fn body_single() -> usize {
    use crate::util::HashMap;
    let mut m = HashMap::new();
    m.insert(1u8, 2u8);
    m.len()
}

fn main() {
    let mut m: HashMap<u8, u8> = HashMap::new();
    m.insert(1, 2);
    println!("{}", util::count(&m));
    let r = deep::make();
    println!("{:?} {}", r.borrow(), Rc::strong_count(&r));
    println!("{:?}", deep::set());
    println!("{}", local_wins::get());
    println!("{} {}", parent::shared(), parent::child::again());
    println!("{} {}", body_glob(), body_single());
    // The glob also brings the `as _` trait import: `write!` into a String.
    let mut s = String::new();
    write!(s, "{}-{}", 1, 2).unwrap();
    println!("{s}");
}
