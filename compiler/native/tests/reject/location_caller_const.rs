// expect: L0630
// Valid Rust: `Location::caller()` is a const fn call. Lust's const
// evaluator has no call target for it, so a const item built from it
// rejects (fx-attrs; the runtime form works).
use std::panic::Location;

const SITE: &Location = Location::caller();

fn main() {
    println!("{}", SITE.line());
}
