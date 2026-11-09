// expect: L0112
// ExitStatus has equality but no ordering.
use std::process::Command;

fn main() {
    let a = Command::new("true").status().unwrap();
    let b = Command::new("true").status().unwrap();
    println!("{}", a < b);
}
