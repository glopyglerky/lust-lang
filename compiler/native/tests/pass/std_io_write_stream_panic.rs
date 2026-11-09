// panic: in fmt
// A panic inside a fmt impl during `write!` on the streams: the parts
// written before it stay written. Stderr is unbuffered, so its
// partial text is out; on stdout the text through the last newline
// is out and the held tail is lost at the abort.
use std::fmt;
use std::io::{self, Write};

struct P;

impl fmt::Display for P {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a\nb{}", 1)?;
        panic!("in fmt")
    }
}

fn main() {
    let _ = write!(io::stdout(), "start ");
    let _ = write!(io::stderr(), "e1 {} ", 2);
    let _ = writeln!(io::stderr(), "e2 {}", P);
    println!("never");
}
