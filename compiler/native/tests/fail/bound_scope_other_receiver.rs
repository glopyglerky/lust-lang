// expect: L0104
// rustc E0599: a bound `T: Add` puts `add` in scope for `T` only, not for `1i32.add(2)`.
fn f<T: std::ops::Add<Output = T>>(x: T, y: T) -> (T, i32) {
    (x.add(y), 1i32.add(2))
}
fn main() {
    println!("{:?}", f(1u8, 2).1);
}
