// expect: L0110
fn main() {
    let x: u8 = 1;
    println!("{}", matches!(x, 1.0..=2.0));
}
