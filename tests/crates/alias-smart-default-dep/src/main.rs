use dmap::{Counter, Map, Shared};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

type Dir = Rc<RefCell<Map<String, String>>>;
type Count = Arc<Mutex<Counter>>;
type Boxed = Box<Map<u8, u8>>;
type Plain = Rc<Counter>;

trait Cloneable {
    fn clone_(&self) -> Self;
}

impl Cloneable for Dir {
    fn clone_(&self) -> Self {
        let m = self.borrow();
        let mut out = Map::default();
        for (k, v) in m.keys.iter().zip(&m.vals) {
            out.insert(k.clone(), v.clone());
        }
        Rc::new(RefCell::new(out))
    }
}

fn add(dirs: &mut Vec<Dir>, name: &str) {
    let dir = Dir::default();
    dir.borrow_mut().insert(name.to_string(), "x".to_string());
    dirs.push(dir);
}

fn main() {
    let mut dirs = Vec::new();
    add(&mut dirs, "a");
    let mut on_added = |n: &str| {
        let dir = Dir::default();
        dir.borrow_mut().insert(n.to_string(), n.to_string());
        dirs.push(dir);
    };
    on_added("b");
    let copy = dirs[1].clone_();
    println!("{} {} {}", dirs.len(), copy.borrow().len(), Rc::ptr_eq(&dirs[1], &copy));

    let c = Count::default();
    c.lock().unwrap().n += 3;
    let made: Vec<Count> = (0..2).map(|_| Count::default()).collect();
    println!("{:?} {}", c.lock().unwrap(), made.len());

    let b = Boxed::default();
    let p = Plain::default();
    println!("{} {:?}", b.len(), p);

    let s = Shared::default();
    s.borrow_mut().insert("k".to_string(), 7);
    let t = dmap::Shared::default();
    println!("{} {}", s.borrow().len(), t.borrow().len());

    // As fn values.
    let f: fn() -> Dir = Dir::default;
    let g = None.unwrap_or_else(Count::default);
    let h = true.then(Shared::default).unwrap();
    let i: Option<Boxed> = Some(()).map(|()| Boxed::default());
    println!("{} {:?} {} {}", f().borrow().len(), g.lock().unwrap().n, h.borrow().len(), i.unwrap().len());
}
