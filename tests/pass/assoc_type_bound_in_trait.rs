trait Tr { type Out: std::fmt::Debug; fn get(&self) -> Self::Out; }
impl Tr for u8 { type Out = u32; fn get(&self) -> u32 { *self as u32 + 1 } }
fn main() { println!("{:?}", 1u8.get()); }
