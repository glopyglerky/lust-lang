// A user io::Write type that never becomes `dyn Write` does not block a
// `Box<dyn Write>` of stdout: the eager vtable's write_fmt slot for the
// user type never runs.
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
fn take(w: &mut dyn Write) {
    write!(w, "t{}\n", 3).unwrap();
}
fn main() {
    let mut s = Sink(0);
    s.write_all(b"abc").unwrap();
    let mut b: Box<dyn Write> = Box::new(std::io::stdout());
    writeln!(b, "x {:>3}", 2).unwrap();
    take(&mut *b);
    let mut v: Vec<u8> = Vec::new();
    take(&mut v);
    println!("{:?} {}", v, s.0);
}
