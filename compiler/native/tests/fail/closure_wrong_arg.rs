// expect: L0110
fn main() {
    let f = |x: u8| x as u16;
    println!("{}", f(300u16));
}
