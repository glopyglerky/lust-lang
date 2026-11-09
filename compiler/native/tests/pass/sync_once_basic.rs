// std::sync::Once (std source overlay): call_once runs its closure once,
// in statics, struct fields, and across threads; is_completed and Debug.
use std::sync::Once;
use std::sync::atomic::{AtomicU32, Ordering};

static INIT: Once = Once::new();
static COUNT: AtomicU32 = AtomicU32::new(0);

fn setup() -> u32 {
    INIT.call_once(|| {
        COUNT.fetch_add(1, Ordering::SeqCst);
    });
    COUNT.load(Ordering::SeqCst)
}

struct Holder {
    once: Once,
    hits: u32,
}

fn main() {
    println!("{}", INIT.is_completed());
    println!("{} {}", setup(), setup());
    println!("{}", INIT.is_completed());
    let mut h = Holder { once: Once::new(), hits: 0 };
    let mut n = 0;
    for _ in 0..3 {
        h.once.call_once(|| n += 1);
    }
    h.hits += n;
    println!("{} {:?}", h.hits, h.once);
    let threads: Vec<_> = (0..4).map(|_| std::thread::spawn(setup)).collect();
    for t in threads {
        print!("{} ", t.join().unwrap());
    }
    println!();
}
