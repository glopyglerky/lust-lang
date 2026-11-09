// A Display impl that fails inside `println!`: the pieces before it are
// written (and lost with the partial line), then core panics.
// Abort mode only (unwinding flushes the tail): panics with a formatting trait implementation returned an error when the underlying stream did not
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
    eprintln!("e1 {}", 5);
    println!("done\nhalf {}", Bad);
}
