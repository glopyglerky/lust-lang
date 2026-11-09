// expect: L0507
// A child module sees its parent's private glob ambiguity through
// `use super::*` (rustc E0659).
mod a {
    pub struct X;
    impl X {
        pub fn v() -> u32 {
            1
        }
    }
}
mod b {
    pub struct X;
    impl X {
        pub fn v() -> u32 {
            2
        }
    }
}
mod c {
    use crate::a::*;
    use crate::b::*;
    pub mod d {
        use super::*;
        pub fn f() -> u32 {
            X::v()
        }
    }
}
fn main() {
    println!("{}", c::d::f());
}
