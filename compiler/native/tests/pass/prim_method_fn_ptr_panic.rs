// panic: argument of integer square root cannot be negative
// A `#[track_caller]` builtin method called through a fn pointer panics
// at its `int_impl!` line in core, as rustc's fn pointer shim reports.
fn main() {
    let f: fn(i32) -> i32 = i32::isqrt;
    let n = std::env::args().count() as i32;
    println!("{}", f(16 - 20 * n));
}
