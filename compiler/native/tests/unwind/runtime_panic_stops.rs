// A std helper that panics writes nothing more: the buffer of a short
// encode_utf8/encode_utf16 stays as it was, a misused DebugMap prints
// only what Rust prints before its panic, and an atomic store or CAS
// with a bad ordering leaves the value alone.
use std::fmt;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicU32, Ordering};

struct M(u32);
impl fmt::Debug for M {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut m = f.debug_map();
        m.key(&1);
        if self.0 == 1 {
            m.key(&2);
        }
        if self.0 == 2 {
            m.finish()?;
        }
        m.value(&3);
        m.value(&4);
        m.finish()
    }
}

fn ord(n: usize, bad: Ordering) -> Ordering {
    if n > 0 { bad } else { Ordering::Relaxed }
}

fn main() {
    let n = std::env::args().count();
    let mut b = [7u8; 1];
    let r = catch_unwind(AssertUnwindSafe(|| 'é'.encode_utf8(&mut b).len()));
    println!("utf8 {:?} {}", b, r.is_err());
    let mut w = [7u16; 1];
    let r = catch_unwind(AssertUnwindSafe(|| '𝄞'.encode_utf16(&mut w).len()));
    println!("utf16 {:?} {}", w, r.is_err());
    for k in 1..4 {
        let r = catch_unwind(|| println!("{:?}", M(k)));
        println!("|map {k} {}", r.is_err());
        let r = catch_unwind(|| print!("{:#?}", M(k)));
        println!("|alt map {k} {}", r.is_err());
        let r = catch_unwind(|| format!("{:?}", M(k)));
        println!("format map {k} {:?}", r.ok());
    }
    let a = AtomicU32::new(1);
    let r = catch_unwind(|| a.store(5, ord(n, Ordering::Acquire)));
    println!("store {} {}", a.load(Ordering::SeqCst), r.is_err());
    let r = catch_unwind(|| a.store(6, ord(n, Ordering::AcqRel)));
    println!("store {} {}", a.load(Ordering::SeqCst), r.is_err());
    let r = catch_unwind(|| a.compare_exchange(1, 8, Ordering::SeqCst, ord(n, Ordering::Release)));
    println!("cas {} {}", a.load(Ordering::SeqCst), r.is_err());
    let r = catch_unwind(|| a.compare_exchange_weak(1, 9, Ordering::SeqCst, ord(n, Ordering::AcqRel)));
    println!("cas weak {} {}", a.load(Ordering::SeqCst), r.is_err());
}
