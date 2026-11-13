// expect: L0110
// Vec and Option elements must be Sized (rustc E0277).
use std::path::Path;
fn main() {
    let v: Vec<str> = Vec::new();
    let o: Option<Path> = None;
}
