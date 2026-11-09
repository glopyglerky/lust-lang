// Deref coercion through `MutexGuard`, the `RwLock` guards, `Ref`, and `RefMut`
// at call arguments, `let` annotations, and returns; fields through a guard.
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, RwLock};
struct L {
    n: Vec<i32>,
    children: Option<HashMap<u32, u32>>,
}
fn take(l: &mut L) {
    l.n.push(l.n.len() as i32);
}
fn peek(l: &L) -> usize {
    l.n.len()
}
fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}
fn first<'a>(g: &'a MutexGuard<'_, L>) -> &'a L {
    g
}
fn main() {
    let m = Mutex::new(L { n: vec![], children: None });
    {
        let mut g = lock(&m);
        take(&mut g);
        take(&mut g);
        println!("{}", peek(&g));
        g.children.get_or_insert_with(|| HashMap::new()).insert(1, 2);
        let r: &mut L = &mut g;
        r.n.push(9);
        let s: &L = &g;
        println!("{:?} {}", s.n, first(&g).children.as_ref().map_or(0, |c| c.len()));
    }
    let rw = RwLock::new(L { n: vec![1], children: None });
    take(&mut rw.write().unwrap());
    println!("{}", peek(&rw.read().unwrap()));
    let mut w = rw.write().unwrap();
    take(&mut w);
    println!("{}", peek(&w));
    drop(w);
    let c = RefCell::new(L { n: vec![], children: None });
    take(&mut c.borrow_mut());
    let mut b = c.borrow_mut();
    take(&mut b);
    println!("{}", peek(&b));
    drop(b);
    println!("{}", peek(&c.borrow()));
}
