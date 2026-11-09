// expect: L0617
// rustc E0310: a field naming In<T> inside a dyn type needs T: 'static declared.
struct In<T: 'static>(Option<T>);
struct S<T>(Box<dyn Fn(In<T>)>);
fn main() {}
