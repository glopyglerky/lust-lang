// `self.0.write(b)` on Arc<dyn Rwc>: Arc implements io::Write only for
// Arc<File>, and dyn Rwc's own method is an object (inherent) candidate.
use std::io::Write;
use std::sync::Arc;
trait Rwc: Send + Sync {
    fn write(&self, b: &[u8]) -> std::io::Result<usize>;
    fn flush(&self) -> std::io::Result<()>;
}
struct S;
impl Rwc for S {
    fn write(&self, b: &[u8]) -> std::io::Result<usize> {
        Ok(b.len())
    }
    fn flush(&self) -> std::io::Result<()> {
        Ok(())
    }
}
struct W(Arc<dyn Rwc>);
impl Write for W {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        self.0.write(b)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
}
fn main() {
    let mut w = W(Arc::new(S));
    println!("{}", w.write(b"abc").unwrap());
    w.flush().unwrap();
}
