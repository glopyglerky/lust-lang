// expect: L0110
// consume has no default body in io::BufRead.
use std::io::{self, BufRead, Read};

struct R;

impl Read for R {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        Ok(0)
    }
}

impl BufRead for R {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        Ok(b"")
    }
}

fn main() {
    let mut r = R;
    let mut s = String::new();
    let _ = r.read_line(&mut s);
}
