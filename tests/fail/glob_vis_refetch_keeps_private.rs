// expect: L0509
// `h0` gets `i2::X` through its private glob of `c` and then through
// `pub use crate::h1::*`. rustc first records the wider visibility, but
// `c`'s own binding changes later (it also globs `h1`), and the re-sent
// binding replaces `h0`'s, so `h0::X` is a private import (E0603). Which
// binding wins depends on that order, so lust reports the order rule.
mod i2 {
    pub struct X {}
}
mod h0 {
    use self::c::*;
    pub use crate::h1::*;
    pub mod c {
        pub(crate) use crate::i2::*;
        pub use crate::h1::*;
    }
}
mod h1 {
    pub use crate::i2::X;
}
fn main() {
    let _ = h0::X {};
    println!("ok");
}
