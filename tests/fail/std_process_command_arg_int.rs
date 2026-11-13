// expect: L0612
// Command arguments are OS strings, not integers.
use std::process::Command;

fn main() {
    let _ = Command::new("true").arg(5).status();
}
