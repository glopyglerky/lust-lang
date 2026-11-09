// expect: L0110
// rustc E0308: one type is more general than the other. The cast's
// pointer returns its input's borrow; `take`'s `T` is fixed outside it.
fn take<T>(f: fn(&u8) -> T, x: &u8) -> T {
    f(x)
}
fn main() {
    let x = 4u8;
    println!("{}", take::<&u8>((|a| a) as fn(&u8) -> &u8, &x));
}
