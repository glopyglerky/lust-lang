// expect: L0308
// write_all takes &mut self.
use std::io::Write;
fn main() {
    let s = std::io::stdout();
    let _ = s.write_all(b"x");
}
