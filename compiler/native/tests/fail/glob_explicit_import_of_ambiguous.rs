// expect: L0507
// An explicit import of a glob-ambiguous name is an error even when
// nothing uses it (rustc E0659).
mod i2 {
    pub struct X {}
}
mod i3 {
    pub struct X {}
}
mod h3 {
    pub use crate::i2::*;
    pub use crate::i3::*;
}
mod h1 {
    #[allow(unused_imports)]
    use crate::h3::X;
}
fn main() {
    println!("1");
}
