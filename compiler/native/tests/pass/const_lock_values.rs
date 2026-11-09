use std::sync::{Condvar, Mutex, OnceLock, RwLock};
const M: Mutex<i32> = Mutex::new(1);
const R: RwLock<i32> = RwLock::new(2);
const O: OnceLock<i32> = OnceLock::new();
const C: Condvar = Condvar::new();
const V: Mutex<Vec<i32>> = Mutex::new(Vec::new());
fn main() {
    let m = M;
    println!("{}", m.into_inner().unwrap());
    let r = R;
    println!("{}", r.into_inner().unwrap());
    let o = O;
    println!("{}", o.get().is_none());
    println!("{}", o.get_or_init(|| 7));
    let c = C;
    c.notify_one();
    println!("cv");
    let v = V;
    v.lock().unwrap().push(3);
    println!("{}", v.lock().unwrap().len());
    println!("{}", *M.lock().unwrap());
}
