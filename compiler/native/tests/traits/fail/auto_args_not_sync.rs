// rustc-error: E0277
// lust-expect: L0612
// ARCH 2.7 table: std::env::Args is neither Send nor Sync.

fn need<T: Sync>(_t: &T) {}
fn main() { let v = std::env::args(); need(&v); println!("ok"); }
