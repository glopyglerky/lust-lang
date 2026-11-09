// expect: L0631
// rustc E0658: `From::from` is only conditionally const (unstable), so `u64::from(x)` cannot run in a const fn.
const fn widen(x: u8) -> u64 {
    u64::from(x)
}
fn main() {
    println!("{}", widen(3));
}
