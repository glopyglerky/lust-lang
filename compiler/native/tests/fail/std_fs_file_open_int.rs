// expect: L0612
// File paths are not integers.
use std::fs::File;

fn main() {
    let _ = File::open(5);
}
