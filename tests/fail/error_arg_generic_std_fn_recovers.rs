// expect: L0101
// rustc E0425: the names are unresolved. An `{error}` argument to a
// generic std fn (`AsRef<Path>`) once made lust try every impl and report
// each one (20,000 L0613 in 113 s); now the body stops at the real errors.
use std::path::Path;
fn main() {
    std::process::Command::new("ln").arg("-s").arg("t.txt").arg(&link).status().unwrap();
    let r3 = std::fs::read_link(&l2).unwrap();
    let c = std::fs::canonicalize(&link).unwrap();
    match std::fs::read_link(&target) {
        Ok(_) => println!("ok?"),
        Err(e) => println!("{:?} {}", e.kind(), e),
    }
    match Path::new("a\0b").read_link() {
        Ok(_) => println!("ok?"),
        Err(e) => println!("{:?} {}", e.kind(), e),
    }
}
