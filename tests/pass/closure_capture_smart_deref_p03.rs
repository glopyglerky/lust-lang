use std::sync::{Arc, Mutex};
fn main() {
    let m = Arc::new(Mutex::new(vec![1u8]));
    let mut g = m.lock().unwrap();
    let mut push = |x| g.push(x);
    push(2); push(3);
    println!("{:?}", *g);
    let t = (Arc::new(5), 6);
    let f = || *t.0 + t.1;
    println!("{}", f());
}
