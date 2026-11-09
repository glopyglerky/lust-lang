// expect: L0630
// rustc E0080: `h` is a u8 (from the return), so the loop overflows at
// 250 + 10 even though the final value would fit.
const X: u8 = {
    let mut h = 0;
    let mut i = 0;
    while i < 30 {
        h += 10;
        i += 1;
    }
    h -= 200;
    h
};
fn main() {
    println!("{X}");
}
