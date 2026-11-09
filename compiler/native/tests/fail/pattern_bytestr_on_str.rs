// expect: L0110
fn main() {
    let s: &str = "abc";
    println!("{}", matches!(s, b"abc"));
}
