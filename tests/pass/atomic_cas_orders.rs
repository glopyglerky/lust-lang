use std::sync::atomic::{AtomicU32, AtomicBool, Ordering};
// Every legal success/failure pair, and runtime-chosen orderings.
fn main() {
    let a = AtomicU32::new(0);
    let ords = [Ordering::Relaxed, Ordering::Release, Ordering::Acquire, Ordering::AcqRel, Ordering::SeqCst];
    let fails = [Ordering::Relaxed, Ordering::Acquire, Ordering::SeqCst];
    let mut n = 0;
    for s in ords {
        for f in fails {
            let r = a.compare_exchange(n, n + 1, s, f);
            let w = a.compare_exchange_weak(n + 7, 0, s, f);
            println!("{:?} {:?} {:?} {:?}", s, f, r, w);
            n += 1;
        }
    }
    let b = AtomicBool::new(false);
    for o in ords {
        println!("{} {}", b.fetch_xor(true, o), b.fetch_or(false, o));
        std::sync::atomic::compiler_fence(if o == Ordering::Relaxed { Ordering::SeqCst } else { o });
    }
    println!("{:?}", a);
}
