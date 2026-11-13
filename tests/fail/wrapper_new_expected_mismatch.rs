// expect: L0110
// The argument is checked against the expected value type, and `u32`
// does not coerce to `u64` (rustc E0308).
fn main() {
    let m: std::sync::Mutex<Option<u64>> = std::sync::Mutex::new(Some(5u32));
    let _ = m;
}
