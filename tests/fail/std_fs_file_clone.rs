// expect: L0104
// File is not Clone (try_clone makes a new handle with an error).
use std::fs::File;

fn main() {
    let f = File::open("a").unwrap();
    let _g = f.clone();
}
