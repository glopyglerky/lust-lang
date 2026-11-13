// expect: L0110
// read_line takes &mut String, not &mut Vec<u8>.
use std::io::BufRead;

fn main() {
    let mut s: &[u8] = b"a\n";
    let mut v: Vec<u8> = Vec::new();
    let _ = s.read_line(&mut v);
}
