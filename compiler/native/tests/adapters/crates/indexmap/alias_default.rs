// indexmap 2.14.0: `Alias::default()` and `Alias::default` as a fn value for aliases of Rc, Arc, and Box over an IndexMap or IndexSet (ts_goport snapshotfs.rs `CachedDirectory::default()`, fx-aliasdef).
use indexmap::{IndexMap, IndexSet};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

type CachedDirectory = Rc<RefCell<IndexMap<String, String>>>;
type Shared = Arc<Mutex<IndexSet<u32>>>;
type Boxed = Box<IndexMap<u8, Vec<u8>>>;

trait Cloneable {
    fn clone_(&self) -> Self;
}

impl Cloneable for CachedDirectory {
    fn clone_(&self) -> Self {
        Rc::new(RefCell::new(self.borrow().clone()))
    }
}

fn main() {
    let mut dirs: Vec<CachedDirectory> = Vec::new();
    let mut on_added = |path: &str, base: &str| {
        let dir = CachedDirectory::default();
        dir.borrow_mut().insert(path.to_string(), base.to_string());
        dirs.push(dir);
    };
    on_added("/a", "x");
    on_added("/b", "y");
    let c = dirs[0].clone_();
    c.borrow_mut().insert("/c".to_string(), "z".to_string());
    println!("{:?} {:?}", dirs[0].borrow(), c.borrow());
    let s = Shared::default();
    s.lock().unwrap().insert(4);
    let b = Boxed::default();
    println!("{:?} {:?}", s.lock().unwrap(), b);
    let f: fn() -> CachedDirectory = CachedDirectory::default;
    let g = true.then(Shared::default).unwrap();
    let h = None.unwrap_or_else(Boxed::default);
    println!("{} {} {}", f().borrow().len(), g.lock().unwrap().len(), h.len());
}
