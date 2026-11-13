// A primitive `#[track_caller]` method item called through an `F: Fn`
// bound runs in its fn item shim: the panic reports
// core/src/ops/function.rs at `Fn::call` (79:5).
fn ap_fn<F: Fn(u32) -> u32>(f: F, x: u32) -> u32 {
    f(x)
}
fn ap_once<F: FnOnce(i32) -> i32>(f: F, x: i32) -> i32 {
    f(x)
}
fn ap_ref<F: Fn(u32) -> u32>(f: &F, x: u32) -> u32 {
    f(x)
}
fn main() {
    println!("{}", ap_fn(u32::ilog2, 8));
    println!("{}", ap_once(i32::isqrt, 16));
    println!("{}", ap_once(&i32::isqrt, 25));
    println!("{}", ap_ref(&u32::ilog2, 0));
}
