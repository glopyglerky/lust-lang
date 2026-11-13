// expect: L0110
// to_str returns Option<&str>.
use std::path::Path;
fn main() {
    let s: &str = Path::new("a").to_str();
}
