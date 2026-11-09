// expect: L0641
// Valid Rust; lust rejects: std's write_fmt calls write once per format
// piece, and lust's write_fmt writes the whole text at once, so a user
// writer behind `dyn Write` would see other calls.
use std::io::Write;
struct Sink(usize);
impl Write for Sink {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        self.0 += 1;
        println!("w{} {}", self.0, b.len());
        Ok(b.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn main() {
    let mut out: Box<dyn Write> = Box::new(std::io::stdout());
    writeln!(out, "start").unwrap();
    let mut b: Box<dyn Write> = Box::new(Sink(0));
    write!(b, "a{}b", 1).unwrap();
}
