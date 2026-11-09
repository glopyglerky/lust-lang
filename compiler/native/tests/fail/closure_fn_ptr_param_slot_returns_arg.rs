// expect: L0733
// rustc: lifetime may not live long enough. `T` is `&'x u8` from the
// local `x`, not the pointer's input lifetime, so the closure cannot
// return its argument there; the second slot is the input's.
fn take<T>(f: fn(&u8) -> (T, &u8), t: T) -> fn(&u8) -> (T, &u8) {
    let _ = t;
    f
}
fn main() {
    let x = 4u8;
    let f = take(|a| (a, a), &x);
    let y = 2u8;
    println!("{:?}", f(&y));
}
