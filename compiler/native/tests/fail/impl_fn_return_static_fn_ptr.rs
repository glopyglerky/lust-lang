// expect: L0735
// rustc: implementation of `Fn` is not general enough.
static G: u8 = 9;
fn gg(_a: &u8) -> &'static u8 {
    &G
}
fn mk() -> impl Fn(&u8) -> &u8 {
    let f: fn(&u8) -> &'static u8 = gg;
    f
}
fn main() {
    println!("{}", mk()(&2));
}
