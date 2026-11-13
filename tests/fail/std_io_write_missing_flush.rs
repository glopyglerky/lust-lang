// expect: L0110
// flush has no default body in io::Write.
use std::io::Write;
struct S;
impl Write for S {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        Ok(b.len())
    }
}
fn main() {
    let mut s = S;
    let _ = s.write_all(b"x");
}
