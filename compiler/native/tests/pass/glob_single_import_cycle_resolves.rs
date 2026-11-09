// Two single imports of `x` that read each other's module. `h1`'s
// pending import blocks `h2`'s only if it could still bind `x`: its
// source `h2` has only a private glob `x`, which `h1` cannot see, so
// `h2`'s import reads `h1`'s glob `x` first, and `h1`'s import then
// reads `h2`'s explicit one (rustc prints 10 10).
mod h1 {
    pub(crate) use crate::h2::x;
    pub use crate::i1::*;
}
mod h2 {
    use crate::i3::*;
    use crate::h1::*;
    pub(crate) use crate::h1::x;
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
fn main() {
    println!("{} {}", h1::x(), h2::x());
}
