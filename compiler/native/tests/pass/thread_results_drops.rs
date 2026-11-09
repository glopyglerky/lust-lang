use std::thread;
use std::sync::atomic::{AtomicUsize, Ordering};
static DROPS: AtomicUsize = AtomicUsize::new(0);
struct D(u32);
impl Drop for D { fn drop(&mut self) { DROPS.fetch_add(1, Ordering::SeqCst); } }
fn main() {
    let hs: Vec<_> = (0..5u32).map(|i| thread::spawn(move || { let v: Vec<u32> = (0..i).collect(); (i, v, D(i)) })).collect();
    let mut sum = 0;
    for h in hs { let (i, v, d) = h.join().unwrap(); sum += i + v.len() as u32 + d.0; }
    println!("{} {}", sum, DROPS.load(Ordering::SeqCst));
    let s = String::from("moved");
    let h = thread::spawn(move || s + "!");
    println!("{} {}", h.join().unwrap(), thread::panicking());
    let h = thread::spawn(|| D(9));
    let t = h.thread().clone();
    let d = h.join().unwrap();
    println!("{:?} {} {}", t.id(), d.0, DROPS.load(Ordering::SeqCst));
    thread::yield_now();
    let big = thread::spawn(|| [7u64; 9]).join().unwrap();
    println!("{:?}", big);
}
