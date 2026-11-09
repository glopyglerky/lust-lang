// expect: L0110
// rustc E0308: one fn item value has one set of args.
fn id<T>(x: T) -> T {
    x
}
fn main() {
    let f = id;
    f(1u8);
    f(2u16);
}
