// expect: L0617
// rustc E0310: `&'static T` in a `let` annotation needs `T: 'static`.
fn f<T>() -> bool {
    let x: Option<&'static T> = None;
    x.is_none()
}
fn main() { println!("{}", f::<u8>()); }
