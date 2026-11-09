// expect: L0507
// rustc E0659 at module level: the glob brings a user `Option`, and
// `use Option::*` does not shadow the prelude.
mod p { pub enum Option { A, B } }
use p::*;
use Option::*;
fn main() { let _ = A; println!("ok"); }
