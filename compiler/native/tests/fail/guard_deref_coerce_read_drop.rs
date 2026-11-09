// expect: L0311
// A read guard's deref borrows the guard: the coerced reference cannot
// outlive it.
use std::sync::RwLock;
struct L(Vec<i32>);
fn main() {
    let m = RwLock::new(L(vec![1]));
    let g = m.read().unwrap();
    let r: &L = &g;
    drop(g);
    println!("{}", r.0.len());
}
