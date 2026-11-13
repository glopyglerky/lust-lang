// expect: L0735
// rustc E0308: the `let` closure's output `W<'_>` has its own inferred
// lifetime, not its argument's (no reference in the caller body).
static G: u8 = 9;
struct W<'a>(&'a u8);
fn apo<F: FnOnce(&u8) -> W<'_>>(f: F) -> u8 {
    let x = 3u8;
    *f(&x).0
}
fn main() {
    let f = |_a: &u8| W(&G);
    println!("{}", apo(f));
}
