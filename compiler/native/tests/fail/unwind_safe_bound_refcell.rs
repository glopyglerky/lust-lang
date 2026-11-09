// expect: L0612
// A written UnwindSafe bound rejects a closure that captures &RefCell.
use std::panic::UnwindSafe;
fn run<F: FnOnce() + UnwindSafe>(f: F) { let _ = std::panic::catch_unwind(f); }
fn main() {
    let c = std::cell::RefCell::new(1);
    run(|| { *c.borrow_mut() += 1; });
}
