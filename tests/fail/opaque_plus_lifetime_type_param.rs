// expect: L0617
// rustc E0309: the returned `impl Sized + 's` holds a `T`, which nothing bounds by `'s`.
fn h<'s, T>(t: T, q: &'s u8) -> impl Sized + 's {
    let _ = q;
    t
}
fn main() {
    let x = 1u8;
    let _r = h(2u8, &x);
}
