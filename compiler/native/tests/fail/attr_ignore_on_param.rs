// expect: L0524
// rustc error: only lint and cfg attributes are allowed on function parameters.
fn f(#[ignore] x: i64) -> i64 {
    x
}
fn main() {
    println!("{}", f(1));
}
