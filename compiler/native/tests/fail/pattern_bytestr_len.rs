// expect: L0110
fn main() {
    let s: &[u8; 2] = b"ab";
    println!("{}", matches!(s, b"abc"));
}
