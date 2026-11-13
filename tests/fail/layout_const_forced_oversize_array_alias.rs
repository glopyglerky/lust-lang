// expect: L0110
type Bytes = [u8; core::mem::size_of::<[u8; 9223372036854775808]>()];
fn main() {}

fn force_alias_evaluation(_: Bytes) {}
