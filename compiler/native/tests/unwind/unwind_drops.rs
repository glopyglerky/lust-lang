// Destructors run while the stack
// unwinds, in reverse construction order, and execution resumes
// after the catch.
use std::sync::atomic::{AtomicU32, Ordering};

static DROPPED: AtomicU32 = AtomicU32::new(0);

struct Guard(u32);

impl Drop for Guard {
    fn drop(&mut self) {
        DROPPED.fetch_add(self.0, Ordering::SeqCst);
        println!("drop {}", self.0);
    }
}

fn blow_up() {
    let _a = Guard(1);
    let _b = Guard(2);
    panic!("drops-boom");
}

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let r = std::panic::catch_unwind(blow_up);
    println!("resumed err={} dropped_sum={}", r.is_err(), DROPPED.load(Ordering::SeqCst));
}
