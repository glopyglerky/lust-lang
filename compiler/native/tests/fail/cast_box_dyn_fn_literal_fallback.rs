// expect: L0110
// rustc E0271: the closure's `1` falls back to `i32` before the cast is
// checked, so it does not return `u8`.
fn main() {
    let b = Box::new(|| 1) as Box<dyn Fn() -> u8>;
    println!("{}", b());
}
