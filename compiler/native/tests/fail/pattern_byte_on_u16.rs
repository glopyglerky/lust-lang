// expect: L0110
fn main() {
    let c: u16 = 32;
    println!("{}", matches!(c, b' '));
}
