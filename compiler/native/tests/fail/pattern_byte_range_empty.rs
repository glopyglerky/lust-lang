// expect: L0119
fn main() {
    let c: u8 = 1;
    println!("{}", matches!(c, b'z'..=b'a'));
}
