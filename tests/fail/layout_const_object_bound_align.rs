// expect: L0110
// align_of also requires the queried object to have a valid layout.
fn bytes() -> [u8; core::mem::align_of::<[u8; 2305843009213693952]>()] { [0; 1] }
fn main() {}
