// `std::fmt::format(args)` (ts-rust stdio write_fmt): the full path, a module
// path, and the imported fn; the String's capacity is std's.
use std::fmt;
use std::io::{self, Write};
struct Out(Vec<u8>);
impl Write for Out {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.extend_from_slice(buf);
        Ok(buf.len())
    }
    fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> io::Result<()> {
        let s = std::fmt::format(args);
        println!("[{}] cap {}", s, s.capacity());
        self.0.extend_from_slice(s.as_bytes());
        Ok(())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
fn main() {
    let mut o = Out(Vec::new());
    write!(o, "a{}b{:>4}", 1, "x").unwrap();
    write!(o, "plain").unwrap();
    writeln!(o).unwrap();
    let s = fmt::format(format_args!("{:?}-{}", Some(3), 2.5));
    println!("{} {}", s, s.capacity());
    use std::fmt::format;
    println!("{}", format(format_args!("{}", "lit")).capacity());
    println!("{:?}", String::from_utf8(o.0).unwrap());
}
