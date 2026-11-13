// expect: L0306
// Sol review probe `drop_user_generic`; rustc: E0597 `x` does not live long enough
struct Watch<T>(T);
impl<T> Drop for Watch<T> { fn drop(&mut self) {} }
fn main() {
    let value;
    { let x = 1; value = Watch(&x); }
}
