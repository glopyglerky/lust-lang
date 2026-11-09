// expect: L0510
// `h0`'s import of `h1::x` may read `h1`'s glob `x` early, but `h1`'s
// final `x` is its own private import of `h2::x`, which `h0` cannot
// see (rustc E0603). Imports are checked against the final tables.
mod h0 {
    pub use crate::i3::*;
    pub use crate::h1::x;
}
mod i1 {
    pub fn x() -> u32 {
        10
    }
}
mod i3 {
    pub fn x() -> u32 {
        30
    }
}
mod h1 {
    pub use crate::i3::*;
    #[allow(unused_imports)]
    use crate::h2::x;
}
mod h2 {
    pub use crate::i1::*;
}
fn main() {
    println!("{}", crate::h0::x());
}
