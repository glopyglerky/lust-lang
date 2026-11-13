// expect: L0110
// `u32::from_le_bytes` takes exactly `[u8; 4]`.
fn main() {
    println!("{}", u32::from_le_bytes([1, 2, 3]));
}
