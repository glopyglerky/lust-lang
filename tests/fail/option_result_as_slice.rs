// expect: L0104
// rustc E0599: `Result` has no `as_slice` (only `Option` does).
fn main() {
    let r: Result<i32, String> = Ok(1);
    println!("{:?}", r.as_slice());
}
