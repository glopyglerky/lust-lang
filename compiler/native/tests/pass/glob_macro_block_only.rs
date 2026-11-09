// Only the block glob names the macro here.
mod b { macro_rules! k { () => { 2u32 } } pub(crate) use k; }
fn main() { { use b::*; println!("{}", k!()); } }
