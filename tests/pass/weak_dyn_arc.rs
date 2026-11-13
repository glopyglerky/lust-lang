// sync::Weak<dyn Tr>: downgrade, upgrade across threads, counts, ptr_eq, Debug, high-align values.
use std::sync::{Arc, Weak, Mutex};
use std::thread;
trait Job: Send + Sync { fn run(&self) -> u32; }
struct J(u32);
impl Job for J { fn run(&self) -> u32 { self.0 } }
impl Drop for J { fn drop(&mut self) { println!("drop J{}", self.0); } }
#[repr(align(64))]
struct Big(u32);
impl Job for Big { fn run(&self) -> u32 { self.0 } }
struct Back { me: Mutex<Option<Weak<dyn Job>>> }
fn job(n: u32) -> Arc<dyn Job> { Arc::new(J(n)) }
fn main() {
    let a = job(4);
    let w = Arc::downgrade(&a);
    let w2 = w.clone();
    println!("{} {} {}", w.strong_count(), w.weak_count(), Arc::weak_count(&a));
    let h = thread::spawn(move || match w2.upgrade() { Some(j) => j.run(), None => 0 });
    println!("{}", h.join().unwrap());
    println!("{:?} {}", w, Weak::ptr_eq(&w, &w.clone()));
    drop(a);
    println!("{} {}", w.upgrade().is_none(), w.strong_count());
    let b = Arc::new(Big(8));
    let bw = Arc::downgrade(&b);
    let bw: Weak<dyn Job> = bw;
    let back = Back { me: Mutex::new(Some(bw)) };
    println!("{}", back.me.lock().unwrap().as_ref().unwrap().upgrade().unwrap().run());
    println!("{}", Arc::strong_count(&b));
    let n: Weak<dyn Job> = Weak::<J>::new();
    println!("{} {}", n.upgrade().is_none(), n.weak_count());
}
