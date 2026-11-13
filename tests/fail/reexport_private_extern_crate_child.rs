// expect: L0510
// rustc E0365: the root's `extern crate core;` is the extern prelude's
// entry, so a child module's `pub use core` re-exports the private item.
extern crate core;
mod m {
    pub use core as c;
}
fn main() {
    println!("{}", m::c::cmp::min(1, 2));
}
