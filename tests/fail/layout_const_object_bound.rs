// expect: L0110
// The 64-bit object-size bound is exclusive, even before isize::MAX.
fn bytes() -> [u8; core::mem::size_of::<[u8; 2305843009213693952]>()] { loop {} }
fn main() {}
