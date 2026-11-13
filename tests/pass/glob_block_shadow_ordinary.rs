// An ordinary path is not import resolution: the inner block's glob
// shadows the outer glob (prints the inner item).
mod a { pub fn who() -> u32 { 1 } }
mod b { pub fn who() -> u32 { 2 } }
fn main() { use a::*; { use b::*; println!("{}", who()); } }
