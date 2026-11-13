// A reader that reports more bytes than the buffer holds: read_to_end's
// first 32-byte probe slice panics in std's alloc/src/io/read.rs, and a
// BufReader's fill asserts in core/src/io/borrowed_buf.rs (tf-panicloc).
use std::io::{BufRead, BufReader, Read};
struct Over(usize);
impl Read for Over {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        Ok(buf.len() + self.0)
    }
}
fn main() {
    let n = std::env::args().count();
    if n > 5 {
        let mut v = Vec::new();
        Over(n).read_to_end(&mut v).unwrap();
    }
    let mut r = BufReader::with_capacity(8, Over(n));
    let _ = r.fill_buf().unwrap();
}
