// expect: L0104
// rustc E0277: `into` on `&i32` has Self = `&i32`, and `i64` has no
// `From<&i32>`.
fn main() {
    let a = &3i32;
    let x: i64 = a.into();
    println!("{}", x);
}
