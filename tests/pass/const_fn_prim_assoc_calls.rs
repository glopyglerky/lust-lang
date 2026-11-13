// Primitive associated const fns called by path inside a const fn
// (ts-rust bundled.rs const_hash: `u64::from_le_bytes([..])`).
const fn word(b: &[u8; 8]) -> u64 {
    u64::from_le_bytes(*b) ^ u64::from_be_bytes(*b)
}
const fn small(b: [u8; 2]) -> u16 {
    u16::from_ne_bytes(b).rotate_left(3)
}
const W: u64 = word(b"abcdefgh");
const S: u16 = small([1, 2]);
fn main() {
    println!("{} {} {}", W, S, word(b"zzzzzzzz"));
}
