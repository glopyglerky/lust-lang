// feature: core.std_basics
// form: Path parts and io::Write into a Vec, as in ts-rust path and emitter code
use std::io::Write as _;
use std::path::Path;
fn main() {
    let p = Path::new("a/b/c.rs");
    let mut buf: Vec<u8> = Vec::new();
    let _ = write!(buf, "x{}", 1);
    println!("{} {}", p.extension().map(|e| e.len()).unwrap_or(0), buf.len());
}
