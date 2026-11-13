// expect: L0720
// rustc E0034: with `use std::io::Write`, `io::Write::write` and the user
// trait's `write` both apply at the `&mut Vec<u8>` probe.
// lust sees both traits too (L0643 when it would see only the user one).
use std::io::Write;
trait Sink {
    fn write(&mut self, b: u8) -> usize;
}
impl Sink for Vec<u8> {
    fn write(&mut self, b: u8) -> usize {
        self.push(b);
        self.len()
    }
}
fn main() {
    let mut v: Vec<u8> = Vec::new();
    println!("{}", v.write(4));
    v.flush().unwrap();
}
