// expect: L0617
// rustc E0309: an `impl Trait` argument's bound implies nothing.
fn f<'a, T: Clone>(_x: impl Iterator<Item = &'a T>) {}
fn main() { f::<u8>(std::iter::empty()); }
