// expect: L0202
// rustc E0637: `'_` is not a lifetime bound.
fn f<T: '_>(t: T) -> T {
    t
}
fn main() {
    f(1u8);
}
