// expect: L0110
fn bytes<const N: usize>() -> [u8; core::mem::size_of::<[u8; N]>()] { loop {} }
fn main() {}
