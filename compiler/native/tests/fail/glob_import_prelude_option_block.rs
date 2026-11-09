// expect: L0507
// rustc E0659: `use Option::*` in the block is ambiguous between the
// glob-brought user enum and the prelude `Option`.
mod p { pub enum Option { A, B } }
fn main() { { use p::*; use Option::*; let _ = A; } println!("ok"); }
