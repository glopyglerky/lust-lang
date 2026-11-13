// expect: L0110
// rustc E0596: &mut &T does not coerce to &mut T.
fn f<'a, T>(x: &'a mut &'a T) -> &'a mut T {
    x
}
fn main() {}
