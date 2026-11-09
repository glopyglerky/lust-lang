use std::sync::{Arc, Mutex};
pub type Waker = Arc<dyn Fn() -> u32 + Send + Sync>;
struct L { waker: Mutex<Option<Waker>> }
fn main() {
    let l = L { waker: Mutex::new(None) };
    *l.waker.lock().unwrap() = Some(Arc::new(|| 7));
    let w = l.waker.lock().unwrap().clone().unwrap();
    println!("{}", w());
}
