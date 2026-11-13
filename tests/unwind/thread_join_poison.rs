// A thread that panics ends alone: join returns Err with its payload;
// a write guard it held poisons the Mutex (is_poisoned, PoisonError's
// into_inner, clear_poison); unwrap on the Err prints Any { .. }.
use std::sync::{Arc, Mutex};
use std::thread;
struct D(&'static str);
impl Drop for D { fn drop(&mut self) { println!("drop {} {}", self.0, thread::panicking()); } }
fn main() {
    let h = thread::spawn(|| { let _d = D("t1"); panic!("thread boom {}", 1); });
    let r = h.join();
    match &r { Ok(_) => println!("ok"), Err(e) => println!("err {:?} {:?}", e.downcast_ref::<String>(), e.downcast_ref::<&str>()) }
    let m = Arc::new(Mutex::new(5));
    let m2 = Arc::clone(&m);
    let r = thread::spawn(move || { let mut g = m2.lock().unwrap(); *g += 1; panic!("poison"); }).join();
    println!("joined err {}", r.is_err());
    println!("poisoned {}", m.is_poisoned());
    match m.lock() { Ok(g) => println!("ok {}", *g), Err(p) => { let g = p.into_inner(); println!("recovered {}", *g); } }
    m.clear_poison();
    println!("poisoned after clear {}", m.is_poisoned());
    let r = thread::spawn(|| 42).join();
    println!("{:?}", r.unwrap());
    let e = thread::spawn(|| { panic!("unwrap me"); }).join();
    let _: () = e.unwrap();
}
