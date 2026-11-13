// expect: L0308
// push needs a mutable PathBuf.
use std::path::Path;
fn main() {
    let p = Path::new("a").to_path_buf();
    p.push("b");
}
