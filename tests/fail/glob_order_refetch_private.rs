// expect: L0509
// The order of tests/reject/glob_order_refetch_drops_vis.rs where rustc re-fetches
// `prelude`'s binding through `geometry`'s private glob and drops the public one (E0603).
mod shapes { pub struct Square; impl Square { pub fn sides() -> u32 { 4 } } }
mod extra { pub use crate::shapes::*; }
mod geometry { pub use crate::shapes::*; use crate::prelude::*; }
mod prelude { pub use crate::shapes::*; pub(crate) use crate::extra::*; }
fn main() { println!("{}", geometry::Square::sides()); }
