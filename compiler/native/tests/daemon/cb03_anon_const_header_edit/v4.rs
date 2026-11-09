fn g(a: [u8; 3]) -> usize {
    a.len()
}
fn main() {
    println!("{}", g([0; 3]));
}
