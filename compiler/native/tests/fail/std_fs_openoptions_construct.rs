// expect: L0107
// OpenOptions cannot be built with a struct literal.
use std::fs::OpenOptions;

fn main() {
    let _o = OpenOptions { read: true };
}
