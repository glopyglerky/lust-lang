// expect: L0540
// A negative float saturates to 0 in an unsigned cast.
fn main() {
    println!("{}", -1.5f64 as u8 - 1);
}
