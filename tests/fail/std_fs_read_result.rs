// expect: L0110
// read_to_string returns io::Result<String>.
use std::fs;
fn main() {
    let s: String = fs::read_to_string("a");
}
