// expect: L0110
// rustc E0308 (one type is more general than the other): `id::<T>`'s
// T is fixed outside the pointer's binder, so `id` cannot become a
// `for<'a> fn(&'a u8) -> &'a u8`.
fn id<T>(x: T) -> T {
    x
}
fn main() {
    let g: fn(&u8) -> &u8 = id;
    println!("{}", g(&1));
}
