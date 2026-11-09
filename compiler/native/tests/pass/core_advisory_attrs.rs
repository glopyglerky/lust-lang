#[inline]
fn f(x: i64) -> i64 {
    x + 1
}
#[inline(always)]
fn g(x: i64) -> i64 {
    x * 2
}
#[cold]
fn h() -> i64 {
    3
}
#[must_use]
fn m() -> i64 {
    4
}
#[deprecated]
fn old() -> i64 {
    5
}
fn main() {
    println!("{} {} {} {} {}", f(m()), g(2), h(), m(), old());
}
