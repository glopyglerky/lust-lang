// expect: L0735
// rustc E0308 (one type is more general than the other): `Box<dyn Fn(T)>`
// with `T = &'x u8` is not a `Box<dyn for<'r> Fn(&'r u8)>` (R11).
fn mk<T>() -> Box<dyn Fn(T)> {
    Box::new(|_t| ())
}
fn main() {
    let g = mk::<&u8>();
    let h: Box<dyn Fn(&u8)> = g;
    let x = 1u8;
    h(&x);
}
