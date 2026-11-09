// A byte string literal as fs::write contents, through
// `[u8; N]: AsRef<[u8]>`.
fn main() {
    let dir = std::env::temp_dir().join(format!("lust_reject_{}", std::process::id()));
    let _ = std::fs::write(dir, b"x");
}
