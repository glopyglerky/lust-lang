// A writer that reports more bytes than it got: Write::write_all's slice
// panics in std's core/src/io/write.rs (tf-panicloc).
use std::io::Write;
struct Over(usize);
impl Write for Over {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Ok(buf.len() + self.0)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn main() {
    let n = std::env::args().count();
    let mut ok = Over(0);
    ok.write_all(b"abc").unwrap();
    println!("ok");
    let mut o = Over(n + 2);
    o.write_all(b"abc").unwrap();
}
