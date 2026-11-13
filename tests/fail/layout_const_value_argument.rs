// expect: L0110
fn bytes() -> [u8; core::mem::size_of::<u32>(0)] { loop {} }
fn main() {}
