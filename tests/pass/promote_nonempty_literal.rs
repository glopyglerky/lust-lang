// rustc promotes `&[1, 2]` to `'static`: the promoted temporary has static
// storage, so the slice may be returned.
fn f() -> &'static [u8] {
    &[1, 2]
}
fn main() {
    println!("{}", f().len());
}
