trait Tr { type A: ?Sized; fn get(&self) -> &Self::A; }
impl Tr for u8 { type A = str; fn get(&self) -> &str { "hi" } }
fn main() { println!("{}", 1u8.get()); }
