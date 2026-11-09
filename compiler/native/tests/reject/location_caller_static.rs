// expect: L0630
// Valid Rust: `Location::caller()` is a const fn call. Lust's const
// evaluator has no call target for it, so a static built from it
// rejects (fx-attrs; the runtime form works).
use std::panic::Location;

static SITE: &Location = Location::caller();

fn main() {
    println!("{}", SITE.line());
}
