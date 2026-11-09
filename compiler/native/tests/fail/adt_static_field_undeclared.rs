// expect: L0617
// rustc E0310: S's field needs T: 'static, which S does not declare (rustc infers no 'static bound).
struct In<T: 'static>(Option<T>);
struct S<T>(In<T>);
fn main() {
    let _s: S<u8> = S(In(None));
}
