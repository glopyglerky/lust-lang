// expect: L0110
// rustc E0308: one type is more general than the other. `T` is fixed
// outside the pointer's binder, so `id`'s output cannot be its input's.
fn take<T>(f: fn(&u8) -> T, x: &u8) -> T {
    f(x)
}
fn id(a: &u8) -> &u8 {
    a
}
fn main() {
    let x = 4u8;
    println!("{}", take::<&u8>(id, &x));
}
