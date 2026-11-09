// Through `F: FnMut` (or a `&mut F` of an item) the shim runs
// `FnMut::call_mut`: core/src/ops/function.rs:166:5.
fn ap_mut<F: FnMut(i64) -> i64>(mut f: F, x: i64) -> i64 {
    f(x)
}
fn ap_nest<F: FnMut(i64) -> i64>(f: F, x: i64) -> i64 {
    ap_mut(f, x)
}
fn main() {
    let mut g = i64::isqrt;
    println!("{}", ap_nest(i64::isqrt, 81));
    println!("{}", ap_mut(&mut g, 4));
    println!("{}", ap_mut(&mut g, -4));
}
