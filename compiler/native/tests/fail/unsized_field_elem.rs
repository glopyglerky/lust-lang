// expect: L0110
// A Vec element inside a struct field must be Sized (rustc E0277).
use std::path::Path;
struct Bad {
    v: Vec<Path>,
}
fn main() {}
