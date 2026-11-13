// expect: L0510
// Stdio cannot be built with a struct literal.
use std::process::Stdio;

fn main() {
    let _s = Stdio { kind: 0 };
}
