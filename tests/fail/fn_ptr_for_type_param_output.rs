// expect: L0110
// rustc E0308: one type is more general than the other, for a fn
// pointer value as for a fn item.
fn take<T>(f: fn(&u8) -> T, x: &u8) -> T {
    f(x)
}
fn main() {
    let x = 4u8;
    let p: fn(&u8) -> &u8 = |a| a;
    println!("{}", take(p, &x));
}
