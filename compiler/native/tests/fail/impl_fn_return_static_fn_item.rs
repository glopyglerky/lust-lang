// expect: L0735
// rustc: implementation of `FnMut` is not general enough (the fn item
// returns `'static`, the type returns its argument's lifetime).
static G: u8 = 9;
fn stat(_a: &u8) -> &'static u8 {
    &G
}
fn mk() -> impl FnMut(&u8) -> &u8 {
    let f = stat;
    f
}
fn main() {
    let mut h = mk();
    println!("{}", h(&4));
}
