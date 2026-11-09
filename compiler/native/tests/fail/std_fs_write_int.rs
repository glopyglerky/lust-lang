// expect: L0612
// fs::write contents must be AsRef<[u8]>.
fn main() {
    let _ = std::fs::write("a", 5);
}
