// The glob brings the prelude item itself by re-export, so the
// variant glob through its name is not ambiguous.
mod p2 { pub use std::option::Option; }
fn main() { { use p2::*; use Option::*; let _ = Some(1); } println!("ok"); }
