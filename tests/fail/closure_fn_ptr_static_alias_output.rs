// expect: L0733
// rustc: lifetime may not live long enough. The alias `SR` is `&'static u8`, so the pointer's
// output is not its input's borrow.
type SR = &'static u8;
type S = fn(&u8) -> SR;
fn main() {
    let f: S = |a| a;
    let y = 2u8;
    println!("{}", f(&y));
}
