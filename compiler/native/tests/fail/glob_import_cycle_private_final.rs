// expect: L0510
// `h1::X` imports `h3::X`, whose final binding is `h3`'s private import
// of `h1::X` (rustc E0603). The fixpoint first saw `h3`'s glob binding;
// the import is checked again against the final tables.
mod i1 {
    pub struct X {}
}
mod h1 {
    pub(crate) use crate::h2::*;
    pub(crate) use crate::h3::X;
}
mod h2 {
    pub use crate::h3::*;
}
mod h3 {
    pub(crate) use crate::i1::*;
    #[allow(unused_imports)]
    use crate::h1::X;
}
fn main() {
    println!("1");
}
