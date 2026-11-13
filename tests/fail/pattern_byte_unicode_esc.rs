// expect: L0001
fn main() {
    let c: u8 = 1;
    println!("{}", matches!(c, b'\u{61}'));
}
