// expect: L0110
// A pointer has a fixed size, but this is still a generic const operation.
fn bytes<T>() -> [u8; core::mem::size_of::<*const T>()] { loop {} }
fn main() {}
