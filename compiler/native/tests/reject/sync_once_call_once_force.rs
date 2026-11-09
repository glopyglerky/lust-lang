// expect: L0104
// Valid Rust; `Once::call_once_force` and `OnceState` have no lust form
// (library/std/src/sync.rs), so lust rejects the call.
use std::sync::Once;
fn main() {
    let o = Once::new();
    o.call_once_force(|s| println!("{}", s.is_poisoned()));
}
