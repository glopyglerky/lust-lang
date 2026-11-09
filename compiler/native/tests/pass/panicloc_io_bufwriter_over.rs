// BufWriter::flush over a writer that reports too many bytes panics where
// std's BufGuard drop calls copy_within (core/src/slice/index.rs) (tf-panicloc).
use std::io::{BufWriter, Write};
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
    let mut w = BufWriter::new(Over(n + 2));
    w.write_all(b"abc").unwrap();
    println!("buffered {}", w.buffer().len());
    w.flush().unwrap();
}
