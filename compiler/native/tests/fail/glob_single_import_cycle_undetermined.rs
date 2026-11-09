// expect: L0506
// `h1::x` and `h0::x` import each other, and each source also has a
// glob `x` the other can see, so neither import can be determined
// (rustc: "cannot determine resolution for the import", E0432).
mod h1 {
    pub(crate) use crate::h0::x;
    pub use crate::i2::*;
}
mod h0 {
    pub(crate) use crate::h1::x;
    pub use crate::i1::*;
}
mod i1 {
    pub fn x() -> u32 {
        10
    }
}
mod i2 {
    pub fn x() -> u32 {
        20
    }
}
fn main() {
    println!("{}", h1::x());
}
