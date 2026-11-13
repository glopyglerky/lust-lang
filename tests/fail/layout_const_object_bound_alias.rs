// expect: L0110
type Bytes = [u8; core::mem::size_of::<[u8; 2305843009213693952]>()];
fn force_alias_evaluation(_: Bytes) {}
fn main() {}
