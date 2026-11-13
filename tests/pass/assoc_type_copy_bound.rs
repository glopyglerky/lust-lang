trait Foo { type Out: Copy; fn get(&self) -> Self::Out; }
impl Foo for u8 { type Out = u16; fn get(&self) -> u16 { *self as u16 + 2 } }
fn main() { println!("{}", 3u8.get()); }
