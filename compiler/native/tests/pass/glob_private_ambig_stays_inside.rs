// Two private globs that bring different `X` make `X` ambiguous only
// inside `c`: a glob of `c` from outside cannot see that binding, so
// `p::X` is `t::X`.
mod a {
    pub struct X;
}
mod b {
    pub struct X;
}
mod t {
    pub struct X;
    impl X {
        pub fn v() -> u32 {
            3
        }
    }
}
mod c {
    use crate::a::*;
    use crate::b::*;
    pub fn f() -> u32 {
        1
    }
}
mod p {
    pub use crate::c::*;
    pub use crate::t::*;
}
fn main() {
    println!("{}", p::X::v() + p::f());
}
