// A Once whose closure panicked is poisoned: the next call_once panics
// with std's message, and is_completed stays false.
use std::panic;
use std::sync::Once;

static O: Once = Once::new();

fn main() {
    panic::set_hook(Box::new(|_| {}));
    let r = panic::catch_unwind(|| O.call_once(|| panic!("boom")));
    println!("{} {}", r.is_err(), O.is_completed());
    let r2 = panic::catch_unwind(|| O.call_once(|| println!("ran")));
    let msg = r2.err().and_then(|e| e.downcast_ref::<&str>().map(|s| s.to_string()));
    println!("{:?} {}", msg, O.is_completed());
}
