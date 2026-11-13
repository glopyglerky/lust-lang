// expect: L0612
// rustc E0277: the item's bound holds at the value too.
fn cp<T: Copy>(x: T) -> T {
    x
}
fn main() {
    let f = cp::<String>;
    let _ = f;
}
