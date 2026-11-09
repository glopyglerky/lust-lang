// panic: RefCell already borrowed
// RefCell's Debug holds a shared borrow while its value formats: a
// shared borrow inside works, a mutable one panics, as in Rust.
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

struct N {
    me: Option<Rc<RefCell<N>>>,
    v: u8,
    write: bool,
}

impl fmt::Debug for N {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(m) = &self.me {
            if self.write {
                m.borrow_mut().v += 1;
            } else {
                write!(f, "[inner {}] ", m.borrow().v)?;
            }
        }
        write!(f, "N{}", self.v)
    }
}

fn main() {
    let a = Rc::new(RefCell::new(N { me: None, v: 1, write: false }));
    println!("{:?}", a);
    a.borrow_mut().me = Some(a.clone());
    println!("{:?}", a);
    let g = a.borrow_mut();
    println!("{:?}", a);
    drop(g);
    a.borrow_mut().write = true;
    let s = format!("{:?}", a);
    println!("unreachable {}", s);
}
