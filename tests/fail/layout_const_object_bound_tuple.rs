// expect: L0110
// Each field fits below the object-size bound, but their sum does not.
type Bytes = [u8; core::mem::size_of::<([u8; 1152921504606846976], [u8; 1152921504606846976])>()];
fn force_alias_evaluation(_: Bytes) {}
fn main() {}
