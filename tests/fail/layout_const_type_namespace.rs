// expect: L0110
struct std;
fn bytes() -> [u8; std::mem::size_of::<u32>()] { loop {} }
fn main() {}
