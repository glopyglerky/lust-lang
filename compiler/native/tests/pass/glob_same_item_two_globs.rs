// The same item through two globs is one binding, and an explicit
// import of it next to the globs is not ambiguous either.
mod a { pub struct S(pub u16); }
mod b { pub use crate::a::*; }
mod prelude {
    pub use crate::a::*;
    pub use crate::b::*;
    pub use crate::a::S;
}
mod user {
    use crate::prelude::*;
    pub fn get() -> u16 { S(41).0 + 1 }
}
fn main() { println!("{}", user::get()); }
