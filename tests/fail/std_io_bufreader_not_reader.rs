// expect: L0612
// BufReader's inner value must implement Read.
use std::io::BufReader;

fn main() {
    let _r = BufReader::new(5u32);
}
