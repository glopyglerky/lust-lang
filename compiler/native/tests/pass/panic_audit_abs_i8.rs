// `abs` of MIN overflows inside core.
// panic: attempt to negate with overflow
fn main() {
    let n = std::env::args().count() as i8;
    let x = i8::MIN + n - 1;
    println!("{}", (x + 1).abs());
    println!("{}", x.abs());
}
