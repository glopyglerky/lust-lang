// Panics inside a fmt impl during `write!` on stdout and stderr
// unwind out of the stream write: the parts before the panic stay
// written, the stream lock is released, and later writes go on in
// order. A fmt impl error (no write error) panics as std's
// `io::Write::write_fmt` does and unwinds the same way.
use std::fmt;
use std::io::{self, Write};
use std::panic;

struct P;

impl fmt::Debug for P {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "mid-")?;
        panic!("boom");
    }
}

struct Bad;

impl fmt::Display for Bad {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "bad-")?;
        Err(fmt::Error)
    }
}

fn main() {
    let r = panic::catch_unwind(|| {
        let _ = write!(io::stdout(), "pre {:?} post\n", P);
    });
    println!("caught1={}", r.is_err());
    let r2 = panic::catch_unwind(|| {
        let _ = write!(io::stderr(), "epre {} epost\n", Bad);
    });
    println!("caught2={}", r2.is_err());
    let mut lock = io::stdout().lock();
    let _ = writeln!(lock, "final {:?} {}", 42, "x");
}
