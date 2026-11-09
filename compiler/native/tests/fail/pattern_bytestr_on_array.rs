// expect: L0110
fn main() {
    let s: [u8; 3] = *b"abc";
    println!("{}", matches!(s, b"abc"));
}
