// expect: L0733
// rustc E0700: the returned value is another fn's `impl Sized`, which
// captures `'a` (edition 2024), but this fn's `use<>` captures nothing.
fn inner(x: &u8) -> impl Sized {
    *x as usize
}
fn outer<'a>(a: &'a u8) -> impl Sized + use<> {
    inner(a)
}
fn main() {
    let a = 1;
    let _ = outer(&a);
}
