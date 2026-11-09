// expect: L0110
// Output.stdout is a Vec<u8>, not a String.
use std::process::Command;

fn main() {
    let out = Command::new("true").output().unwrap();
    let _s: String = out.stdout;
}
