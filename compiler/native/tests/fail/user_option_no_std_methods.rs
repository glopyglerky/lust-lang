// expect: L0104
// rustc E0599: a user enum named `Option` shadows std's, so it has no
// `unwrap`.
enum Option<T> { Some(T), Nothing }
fn main() {
    let a: Option<i32> = Option::Some(1);
    let _ = a.unwrap();
    let _ = Option::<u8>::Nothing;
}
