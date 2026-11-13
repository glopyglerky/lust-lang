// `HashMap::insert` and `remove` return the map's `V`: the result does not
// hold the `&mut` loan of the map (tsrust T13: a `'static` dyn value
// required the loan to be `'static`).
use std::cell::RefCell;
use std::collections::HashMap;
struct S {
    jobs: RefCell<HashMap<u64, Box<dyn FnOnce()>>>,
}
fn add(s: &S, id: u64, f: Box<dyn FnOnce()>) {
    s.jobs.borrow_mut().insert(id, f);
}
fn take<'a>(m: &mut HashMap<u32, &'a u8>) -> Option<&'a u8> {
    let old = m.remove(&1);
    m.insert(2, &7);
    old
}
fn main() {
    let s = S { jobs: RefCell::new(HashMap::new()) };
    add(&s, 1, Box::new(|| println!("job")));
    let f = s.jobs.borrow_mut().remove(&1).unwrap();
    f();
    let a = 5u8;
    let mut m = HashMap::new();
    m.insert(1, &a);
    println!("{:?} {}", take(&mut m), m.len());
}
