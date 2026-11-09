// A Read impl that reports more bytes than its buffer: core's
// BorrowedCursor::advance_checked assert, at core's site.
use std::io::{copy, sink, Read, Result};
struct B;
impl Read for B {
    fn read(&mut self, b: &mut [u8]) -> Result<usize> {
        Ok(b.len() + 1)
    }
}
fn main() {
    println!("start");
    let _ = copy(&mut B, &mut sink());
}
