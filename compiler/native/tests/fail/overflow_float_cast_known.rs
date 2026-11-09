// expect: L0540
// rustc: `300.0 as u8` saturates to 255, so `+ 1` overflows.
fn main() {
    let f = 300.0f64 as u8;
    println!("{}", f + 1);
}
