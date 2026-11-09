// expect: L0510
// rustc E0365 (deny-by-default pub_use_of_private_extern_crate): an
// `extern crate` item is private, so a `pub use` of it is an error.
extern crate core;
pub use core as c;
fn main() {
    println!("{}", c::cmp::max(1, 2));
}
