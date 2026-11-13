// Each thread unwinds to its own catch points: many threads panic and catch at once,
// nested, and a thread that dies of a panic does not disturb the others.
use std::panic::{catch_unwind, resume_unwind};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
struct D(Arc<AtomicUsize>);
impl Drop for D {
    fn drop(&mut self) { self.0.fetch_add(1, Ordering::SeqCst); }
}
static HOOK: AtomicUsize = AtomicUsize::new(0);
fn main() {
    std::panic::set_hook(Box::new(|_| { HOOK.fetch_add(1, Ordering::SeqCst); }));
    let drops = Arc::new(AtomicUsize::new(0));
    let mut hs = Vec::new();
    for t in 0..8u32 {
        let drops = drops.clone();
        hs.push(std::thread::spawn(move || {
            let mut caught = 0u32;
            for i in 0..500u32 {
                let d = drops.clone();
                let r = catch_unwind(move || {
                    let _g = D(d.clone());
                    let inner = catch_unwind(|| {
                        let _h = D(d.clone());
                        if i % 2 == 0 { panic!("inner {t} {i}"); }
                        i
                    });
                    match inner {
                        Ok(v) => { if v % 3 == 0 { panic!("outer {t} {v}"); } v }
                        Err(e) => resume_unwind(e),
                    }
                });
                if r.is_err() { caught += 1; }
            }
            if t == 7 { panic!("thread 7 dies"); }
            caught
        }));
    }
    let mut total = 0;
    let mut died = 0;
    for h in hs {
        match h.join() {
            Ok(c) => total += c,
            Err(e) => { died += 1; println!("join err: {}", e.downcast_ref::<&str>().unwrap()); }
        }
    }
    println!("caught {total} died {died}");
    println!("drops {} hooks {}", drops.load(Ordering::SeqCst), HOOK.load(Ordering::SeqCst));
}
