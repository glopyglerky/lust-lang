// expect: L0733
// rustc: lifetime may not live long enough (hunt4), FnMut and two inputs.
fn mk() -> impl FnMut(&u8, &u8) -> &'static u8 {
    |a, b| b
}
fn main() {
    let mut f = mk();
    let k = 2u8;
    println!("{}", f(&k, &k));
}
