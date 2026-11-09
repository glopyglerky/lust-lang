// rustc-error: E0277
// lust-expect: L0110
// ARCH 2.2 derives: the derived PartialOrd impl needs Self: PartialEq
// (its supertrait); deriving it alone is rejected.
#[derive(PartialOrd)]
struct S(u8);
fn main() {}
