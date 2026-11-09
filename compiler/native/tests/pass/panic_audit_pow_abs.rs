// `pow` and `abs` overflow inside core: rustc reports core's line for
// the integer type.
// panic: attempt to exponentiate with overflow
fn main() {
    let n = std::env::args().count() as i32;
    println!("{} {}", (-5 * n).abs(), (2 * n).pow(10));
    let x: u16 = 10 + n as u16;
    println!("{}", x.pow(5));
}
