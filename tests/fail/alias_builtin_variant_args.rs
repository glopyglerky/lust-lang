// expect: L0107
// rustc E0109: type arguments go on the alias, not the variant.
type OP = Option<u8>;
fn main() { let o = OP::Some::<u8>(3); println!("{:?}", o); }
