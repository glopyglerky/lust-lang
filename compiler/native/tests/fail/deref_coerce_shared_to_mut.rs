// expect: L0110
// rustc E0308: &&mut T does not coerce to &mut T.
fn f<'a, T>(x: &'a &'a mut T) -> &'a mut T {
    x
}
fn main() {}
