// expect: L0110
type core = u32;
fn bytes() -> [u8; core::mem::size_of::<u32>()] { loop {} }
fn main() {}
