// expect: L0110
// Path is unsized: it cannot be held by value.
use std::path::Path;
fn main() {
    let p: Path = *Path::new("a");
}
