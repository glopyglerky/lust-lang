use std::sync::{Mutex, RwLock};
fn main() {
    let r = RwLock::new(2);
    let w = r.write().unwrap();
    println!("{:?}", r);
    drop(w);
    println!("{:?}", r);
    let m = Mutex::new(3);
    let g = m.lock().unwrap();
    println!("{:?}", m);
    drop(g);
    println!("{:?}", m);
}
