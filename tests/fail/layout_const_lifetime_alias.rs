// expect: L0110
type Ref<'a> = &'a u8;
fn bytes<'a>() -> [u8; core::mem::size_of::<Ref<'a>>()] { loop {} }
fn main() {}
