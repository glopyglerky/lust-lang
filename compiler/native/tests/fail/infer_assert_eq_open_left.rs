// expect: L0610
// rustc E0283: `assert_eq!(make(), 1)` compares `*left == *right` with the
// left type still a variable; `?T: PartialEq<{integer}>` does not fix it
// (hunt6). With the literal on the left, rustc accepts.
trait Pick {
    fn pick() -> Self;
}
impl Pick for i32 {
    fn pick() -> Self {
        10
    }
}
impl Pick for u8 {
    fn pick() -> Self {
        20
    }
}
fn make<T: Pick>() -> T {
    T::pick()
}
fn main() {
    assert_eq!(make(), 1);
}
