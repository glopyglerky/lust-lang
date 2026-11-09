// `Alias::default` for aliases of Rc, Arc, and Box over std and user
// pointees (ts_goport snapshotfs.rs `CachedDirectory::default()`, an alias
// with a user trait impl, called in a closure in a method), next to the
// older spellings of the same value: `<A as Default>::default()`,
// `Default::default()`, and `Rc::default()` with the type written.
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

#[derive(Default, Debug)]
struct Counter {
    n: u32,
}

#[derive(Debug)]
struct Names {
    v: Vec<String>,
}

impl Default for Names {
    fn default() -> Self {
        Names { v: vec!["root".to_string()] }
    }
}

#[derive(Default, Debug)]
struct Pair<T> {
    a: T,
    b: T,
}

type CachedDirectory = Rc<RefCell<HashMap<String, String>>>;
type Locked = Arc<Mutex<BTreeMap<u8, Names>>>;
type Boxed = Box<Pair<i64>>;
type Inner = RefCell<Names>;
type Outer = Rc<Inner>;
type Shared<T> = Rc<RefCell<T>>;
type Count = Arc<Counter>;

trait Cloneable {
    fn clone_(&self) -> Self;
}

impl Cloneable for CachedDirectory {
    fn clone_(&self) -> Self {
        Rc::new(RefCell::new(self.borrow().clone()))
    }
}

struct Builder {
    dirs: RefCell<Vec<CachedDirectory>>,
}

impl Builder {
    fn add(&self, names: &[&str]) {
        let on_added = |name: &str| {
            let dir = CachedDirectory::default();
            dir.borrow_mut().insert(name.to_string(), name.len().to_string());
            self.dirs.borrow_mut().push(dir);
        };
        for n in names {
            on_added(n);
        }
    }
}

fn main() {
    let b = Builder { dirs: RefCell::new(Vec::new()) };
    b.add(&["a", "bc"]);
    let c = b.dirs.borrow()[1].clone_();
    c.borrow_mut().insert("z".to_string(), "0".to_string());
    println!("{} {} {}", b.dirs.borrow().len(), b.dirs.borrow()[1].borrow().len(), c.borrow().len());

    let l = Locked::default();
    l.lock().unwrap().insert(1, Names::default());
    let x = Boxed::default();
    let o = Outer::default();
    o.borrow_mut().v.push("x".to_string());
    let s: Shared<Pair<u8>> = Shared::default();
    s.borrow_mut().b = 9;
    let k = Count::default();
    println!("{:?} {:?} {:?} {:?} {:?}", l.lock().unwrap(), x, o.borrow(), s.borrow(), k);

    // As fn values.
    let f: fn() -> CachedDirectory = CachedDirectory::default;
    let g = None.unwrap_or_else(Locked::default);
    let h = true.then(Outer::default).unwrap();
    let mut slot: Option<Boxed> = None;
    slot.get_or_insert_with(Boxed::default).a += 4;
    let all: Vec<Count> = (0..3).map(|_| Count::default()).collect();
    println!("{} {} {:?} {:?} {}", f().borrow().len(), g.lock().unwrap().len(), h.borrow().v, slot, all.len());

    // Older spellings of the same values.
    let q = <CachedDirectory as Default>::default();
    let r: Locked = Default::default();
    let t: Outer = Rc::default();
    let u = Rc::<RefCell<Names>>::default();
    let w: Boxed = Box::default();
    println!("{} {} {:?} {:?} {:?}", q.borrow().len(), r.lock().unwrap().len(), t.borrow().v, u.borrow().v, w);
}
