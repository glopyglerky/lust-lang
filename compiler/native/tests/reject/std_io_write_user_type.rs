// expect: L0641
// rustc's write_fmt calls the writer's write once per format piece; lust
// formats first and writes once, so a user writer would see other calls.
use std::io::{self, Write};
struct Counter {
    calls: usize,
}
impl Write for Counter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.calls += 1;
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
fn main() {
    let mut c = Counter { calls: 0 };
    let _ = write!(c, "a{}b{}", 1, 2);
    println!("{}", c.calls);
}
