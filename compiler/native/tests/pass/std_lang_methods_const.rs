// The std source methods core makes const (`is_some`, `unwrap`,
// `expect`, `is_ok`, ...) are callable in const fns and const items.
const A: Option<i32> = Some(3);
const B: i32 = A.unwrap();
const C: bool = A.is_some();
const fn f(x: Option<u8>) -> u8 {
    if x.is_none() { 0 } else { x.expect("x") }
}
const fn g(r: Result<u8, u8>) -> bool {
    r.is_ok()
}
const D: u8 = f(Some(9));
fn main() {
    println!("{} {} {} {}", B, C, D, g(Err(1)));
}
