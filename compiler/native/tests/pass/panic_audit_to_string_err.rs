// `to_string` on a failing Display panics inside alloc.
// panic: a Display implementation returned an error unexpectedly: Error
use std::fmt;

struct Bad;

impl fmt::Display for Bad {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        Err(fmt::Error)
    }
}

fn main() {
    println!("start");
    let s = Bad.to_string();
    println!("{}", s);
}
