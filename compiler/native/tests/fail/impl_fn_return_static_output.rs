// expect: L0733
// rustc: lifetime may not live long enough (hunt4). The returned closure
// may not put its argument in the `'static` output.
fn mk() -> impl Fn(&u8) -> &'static u8 {
    |a| a
}
fn main() {
    let f = mk();
    let k = 2u8;
    println!("{}", f(&k));
}
