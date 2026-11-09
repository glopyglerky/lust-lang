// expect: L0612
// rustc E0277: `AssertUnwindSafe<F>` is `FnOnce()` only when `F` takes no arguments.
fn task<F: Fn(u8) -> u8>(f: F) -> u8 {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).unwrap_or(0)
}
fn main() {
    println!("{}", task(|x| x));
}
