use std::io::{BufWriter, Write};
pub struct Writer { w: BufWriter<Box<dyn Write + Send>> }
pub fn new_writer(w: Box<dyn Write + Send>) -> Writer { Writer { w: BufWriter::with_capacity(4096, w) } }
impl Writer {
    pub fn write(&mut self, data: &[u8]) -> Result<(), String> {
        if let Err(err) = write!(self.w, "Content-Length: {}\r\n\r\n", data.len()) { return Err(err.to_string()); }
        if let Err(err) = self.w.write_all(data) { return Err(err.to_string()); }
        self.w.flush().map_err(|err| err.to_string())
    }
}
fn main() { let mut w = new_writer(Box::new(std::io::stdout())); w.write(b"{}").unwrap(); println!(); let mut w2 = new_writer(Box::new(Vec::<u8>::new())); w2.write(b"x").unwrap(); }
