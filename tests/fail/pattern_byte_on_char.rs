// expect: L0110
fn main() {
    let c: char = 'a';
    println!("{}", matches!(c, b'a'));
}
