// Std fns and methods by path as values and UFCS calls (tf-p6 R05): the
// prelude's `drop`, JoinHandle::join, Receiver::recv*, rc::Weak::upgrade,
// RefCell/Cell methods, OnceCell::from, Rc/Arc::default, Vec::leak(collect),
// HashMap::clone with a deref coercion, Hash::hash, an alias's default.
use std::cell::{Cell, OnceCell, RefCell};
use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};
use std::rc::{Rc, Weak};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

// Leaked values stay reachable here, so the leak checker sees them.
static KEPT: Mutex<Vec<usize>> = Mutex::new(Vec::new());

struct D(u8);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
struct R {
    a: u32,
}
type W = RefCell<VecDeque<u32>>;

fn pm() -> &'static HashMap<u32, u32> {
    let m: &'static HashMap<u32, u32> = Box::leak(Box::new(HashMap::from([(1, 2)])));
    KEPT.lock().unwrap().push(m as *const HashMap<u32, u32> as usize);
    m
}

fn main() {
    vec![D(1), D(2)].into_iter().for_each(drop);
    let g: fn(D) = drop;
    g(D(3));
    let w = std::thread::Builder::new().spawn(move || 7u8);
    println!("{}", matches!(w.map(std::thread::JoinHandle::join), Ok(Ok(7))));
    let hs: Vec<JoinHandle<u32>> = (0..3).map(|i| std::thread::spawn(move || i * 2)).collect();
    let v: Vec<u32> = hs.into_iter().map(JoinHandle::join).map(|r| r.unwrap()).collect();
    println!("{v:?}");
    let (tx, rx) = mpsc::channel::<i32>();
    tx.send(4).unwrap();
    let f = Receiver::try_recv;
    println!("{:?} {:?}", f(&rx), Receiver::try_recv(&rx));
    let r = Rc::new(5);
    let ws = vec![Rc::downgrade(&r), Weak::new()];
    let up: Vec<Rc<i32>> = ws.iter().filter_map(Weak::upgrade).collect();
    println!("{} {}", up.len(), Weak::strong_count(&ws[0]));
    let x = Rc::try_unwrap(Rc::new(RefCell::new(1))).ok().map(RefCell::into_inner);
    let c = Cell::new(3u8);
    Cell::set(&c, 4);
    println!("{:?} {}", x, Cell::into_inner(c));
    let oc: Vec<OnceCell<u32>> = vec![1u32, 2].into_iter().map(OnceCell::from).collect();
    println!("{:?}", oc);
    let d: Option<Rc<RefCell<Vec<u8>>>> = (1 != 0).then(|| Rc::default());
    let e: Option<Rc<RefCell<Vec<u8>>>> = (1 != 0).then(Rc::default);
    let a: Arc<String> = Arc::default();
    println!("{:?} {:?} {:?}", d, e, a);
    let s: &'static mut [R] = Vec::leak((0..3).map(|i| R { a: i }).collect());
    KEPT.lock().unwrap().push(s.as_ptr() as usize);
    println!("{} {}", s.len(), s[2].a);
    let m: HashMap<u32, u32> = HashMap::clone(pm());
    let am = Arc::new(vec![1u8, 2]);
    println!("{:?} {:?}", m, Vec::clone(&am));
    let mut h = std::collections::hash_map::DefaultHasher::new();
    Hash::hash(&[1u32, 2][..], &mut h);
    std::hash::Hash::hash(&5u8, &mut h);
    println!("{}", h.finish());
    let wd = W::default();
    println!("{}", wd.borrow().len());
}
