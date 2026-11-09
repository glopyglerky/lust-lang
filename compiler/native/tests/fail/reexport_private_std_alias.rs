// expect: L0510
// rustc E0365: `core` here is the module's private `use std as core;`.
mod m {
    use std as core;
    pub use core as c;
}
fn main() {
    println!("{}", m::c::cmp::min(1, 2));
}
