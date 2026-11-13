// expect: L0110
fn bytes() -> [u8; core::mem::size_of::<[u8; 9223372036854775808]>()] { loop {} }
fn main() {}
