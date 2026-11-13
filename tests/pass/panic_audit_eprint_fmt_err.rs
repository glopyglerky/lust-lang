// The same failure on stderr: stderr has no buffer, so "half pre" shows.
// panic: a formatting trait implementation returned an error when the underlying stream did not
use std::fmt;

struct Bad;

impl fmt::Display for Bad {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("pre")?;
        Err(fmt::Error)
    }
}

fn main() {
    println!("start");
    eprintln!("half {}", Bad);
}
