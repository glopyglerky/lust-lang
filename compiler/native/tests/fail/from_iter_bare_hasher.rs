// expect: L0111
// rustc E0283: nothing fixes the table's hasher `S`.
use std::collections::HashMap;
fn main() { let m = HashMap::from_iter([(1, 2)]); println!("{}", m.len()); }
