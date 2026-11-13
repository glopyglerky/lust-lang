// A std platform module from a non-root file: the platform crate joins.
use std::io::Write;

pub fn go() {
    let mut out = std::io::stdout();
    writeln!(out, "hi {}", 2).unwrap();
}
