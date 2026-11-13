// A struct literal's `..base` takes a space after the comma; a qualified
// path `<T as Trait>::item` prints its angle brackets tight.
// panic: assertion failed: S { x: 1, ..s }.x == <i64 as Default>::default()
struct S {
    x: i64,
    y: i64,
}
fn main() {
    let s = S { x: bb(1), y: 2 };
    let _ = s.y;
    assert!(<Vec<i64>>::new().len() == 0);
    assert!(S { x: 1, ..s }.x == <i64 as Default>::default());
}
#[inline(never)]
fn bb<T>(x: T) -> T {
    x
}
