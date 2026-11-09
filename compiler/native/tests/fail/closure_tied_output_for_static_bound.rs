// expect: L0735
// rustc E0308: `tie` gives the closure the signature `for<'a> fn(&'a u8)
// -> &'a u8`, which is no `Fn(&u8) -> &'static u8`.
static G: u8 = 9;
fn tie<F: Fn(&u8) -> &u8>(f: F) -> F {
    f
}
fn aps<F: Fn(&u8) -> &'static u8>(f: F) -> u8 {
    let x = 3u8;
    *f(&x)
}
fn main() {
    let f = tie(|_a: &u8| &G);
    println!("{}", aps(f));
}
