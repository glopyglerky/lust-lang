// expect: L0733
// rustc: implementation of `FnOnce` is not general enough. The bound's
// `T` is fixed for the call, so `id` cannot return its argument there.
fn tk<T, F: Fn(&u8) -> T>(f: F) -> u8 {
    let y = 1u8;
    f(&y);
    0
}
fn id(a: &u8) -> &u8 {
    a
}
fn main() {
    println!("{}", tk::<&u8, _>(id));
}
