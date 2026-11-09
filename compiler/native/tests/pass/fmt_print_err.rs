// panic: a formatting trait implementation returned an error when the underlying stream did not
// A fmt impl error inside println! panics with io::Write::write_fmt's
// text (W-STD-B).
use std::fmt;

struct Bad;

impl fmt::Display for Bad {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        Err(fmt::Error)
    }
}

fn main() {
    println!("first line");
    println!("{} post", Bad);
    println!("never");
}
