// rustc's pretty printer drops a trailing comma in call, method, array,
// and tuple lists, and keeps the one of a 1-tuple.
// panic: assertion failed: bb(1).min(2) + f(&[1, 2][0], (3, 4).1) + (5,).0 == 0
fn main() {
    assert!(f(&1, 2,) == -1);
    assert!(bb(1).min(2,) + f(&[1, 2,][0], (3, 4,).1,) + (5,).0 == 0);
}
fn f(p: &i64, q: i64) -> i64 {
    *p - q
}
#[inline(never)]
fn bb<T>(x: T) -> T {
    x
}
