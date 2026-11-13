// A poisoned Once panics at the call_once caller (std's call_once is
// #[track_caller]); a poisoned LazyCell panics in std's cell/lazy.rs
// (tf-panicloc).
use std::cell::LazyCell;
use std::sync::Once;
static O: Once = Once::new();
fn main() {
    let n = std::env::args().count();
    let _ = std::panic::catch_unwind(|| O.call_once(|| panic!("first {}", n)));
    let r = std::panic::catch_unwind(|| O.call_once(|| {}));
    println!("{}", r.is_err());
    let c = LazyCell::new(|| -> u8 { panic!("init {}", n) });
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| *c));
    let _ = *c;
}
