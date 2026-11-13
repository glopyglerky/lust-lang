// expect: L0735
// rustc E0308 (one type is more general than the other): a `let`
// closure has its own signature, whose output lifetime is one inferred
// lifetime, so it does not implement `Fn(&u8) -> &u8`.
static G: u8 = 9;
fn ap<F: Fn(&u8) -> &u8>(f: F) -> u8 {
    let x = 3u8;
    *f(&x)
}
fn main() {
    let f = |_a: &u8| &G;
    println!("{}", ap(&f));
}
