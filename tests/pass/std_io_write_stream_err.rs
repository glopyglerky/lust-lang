// panic: a formatting trait implementation returned an error when the underlying stream did not
// A fmt impl error inside `write!` on stdout panics as std's
// `io::Write::write_fmt` does, after the parts before it are written:
// "first\npre bad-" reaches stdout (through the last newline), the
// held tail is lost at the abort, and "post" never formats.
use std::fmt;
use std::io::{self, Write};

struct Bad;

impl fmt::Display for Bad {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "bad-")?;
        Err(fmt::Error)
    }
}

fn main() {
    let _ = writeln!(io::stdout(), "first");
    let _ = write!(io::stdout(), "pre\n{} post", Bad);
    println!("never");
}
