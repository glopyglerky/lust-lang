// expect: L0110
fn bytes() -> [u8; core::mem::size_of::<([u8; 18446744073709551615], u64)>()] { loop {} }
fn main() {}
