// expect: L0110
// A zero-sized aligned field rounds the final size up to the bound.
type Bytes = [u8; core::mem::size_of::<([u8; 2305843009213693951], [u64; 0])>()];
fn force_alias_evaluation(_: Bytes) {}
fn main() {}
