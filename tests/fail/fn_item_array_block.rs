// expect: L0110
// rustc E0308: an array element after the first that is a block coerces to
// the first element's fn item type.
fn f() -> i32 {
    1
}
fn g() -> i32 {
    2
}
fn main() {
    let v = [f, { g }];
    println!("{}", v[1]());
}
