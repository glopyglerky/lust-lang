// expect: L0112
// Path has no Display impl; rustc wants .display().
use std::path::Path;
fn main() {
    let p = Path::new("a/b");
    println!("{}", p);
}
