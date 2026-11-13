// Re-exports of extern crates that rustc accepts: a plain extern prelude
// name, a `pub extern crate`, and `pub(crate)` re-exports of a private
// root item (a private root item reaches the whole crate).
extern crate core;
pub extern crate alloc;
mod m {
    pub use std as s;
    pub(crate) use core as k;
    pub use crate::alloc as a;
}
fn f() -> u8 {
    3
}
mod n {
    pub(crate) use crate::f;
}
fn main() {
    println!("{} {} {}", m::s::cmp::max(1, 2), m::k::cmp::min(4, 5), n::f());
}
