// expect: L0101
// rustc E0425: `set_hook` is in `std::panic`, not in `core::panic`.
use core::panic::*;
fn main() {
    set_hook(Box::new(|_| {}));
}
