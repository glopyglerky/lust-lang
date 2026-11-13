// expect: L0507
// `h0::X` is ambiguous in `h0` (a public glob and a private glob bring
// different items). An explicit import of it from outside `h0` is an
// error in rustc (E0659) even though only one side is visible there.
mod i1 {
    pub struct X {}
}
mod i2 {
    pub struct X {}
}
mod h0 {
    pub use crate::i1::*;
    #[allow(unused_imports)]
    use crate::i2::*;
}
mod h2 {
    #[allow(unused_imports)]
    use crate::h0::X;
}
fn main() {
    println!("1");
}
