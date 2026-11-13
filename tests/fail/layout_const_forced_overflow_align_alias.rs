// expect: L0110
type Bytes = [u8; core::mem::size_of::<([u8; 18446744073709551615], u64)>()];
fn main() {}

fn force_alias_evaluation(_: Bytes) {}
