// expect: L0617
// rustc E0310: a &'static T field needs T: 'static declared.
struct S<T>(Option<&'static T>);
fn main() {
    let _s: S<u8> = S(None);
}
