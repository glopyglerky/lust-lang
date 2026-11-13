// Unsuffixed literals in a const take their type from inference:
// `!0` at u8 is 255, not the i128 -1.
const T: u8 = 1;
const X: bool = !0 > T;
const Y: i64 = (1 << 40) >> 7;
const Z: u8 = !0 >> 4;

fn main() {
    println!("{} {} {}", X, Y, Z);
}
