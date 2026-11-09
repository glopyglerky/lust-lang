// expect: L0110
// fs::exists returns io::Result<bool>.
fn main() {
    let b: bool = std::fs::exists("a");
}
