// expect: L0733
// rustc E0700: the hidden type `&'b u8` names `'b`, which the `use<'a>`
// list does not capture.
fn pick<'a, 'b>(_x: &'a u8, y: &'b u8) -> impl Sized + use<'a> {
    y
}
fn main() {
    let (a, b) = (1, 2);
    let _ = pick(&a, &b);
}
