// expect: L0110
fn main() {
    let s: &[u8] = b"ab";
    println!("{}", matches!(&s, b"ab"));
}
