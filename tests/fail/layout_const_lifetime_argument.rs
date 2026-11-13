// expect: L0110
fn bytes() -> [u8; core::mem::size_of::<'static, u32>()] { loop {} }
fn main() {}
