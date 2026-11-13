// expect: L0110
fn bytes() -> [u8; core::mem::size_of::<*const (u8, str)>()] { [0; 8] }
fn main() {}
