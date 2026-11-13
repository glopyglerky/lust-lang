// expect: L0110
fn bytes<core>() -> [u8; core::mem::size_of::<u32>()] { loop {} }
fn main() {}
