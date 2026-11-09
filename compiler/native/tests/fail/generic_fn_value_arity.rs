// expect: L0107
// rustc E0107: too many generic arguments on a fn value.
fn id<T>(x: T) -> T {
    x
}
fn main() {
    let f = id::<u8, u8>;
    let _ = f;
}
