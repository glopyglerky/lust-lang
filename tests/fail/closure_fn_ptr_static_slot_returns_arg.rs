// expect: L0733
// rustc: lifetime may not live long enough. The second output slot is
// `'static`; only the first is the pointer's input lifetime.
fn main() {
    let f: fn(&u8) -> (&u8, &'static u8) = |a| (a, a);
    let y = 2u8;
    println!("{:?}", f(&y));
}
