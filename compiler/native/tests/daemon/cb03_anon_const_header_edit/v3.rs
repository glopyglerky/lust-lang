fn g(a: [u8; {
    let n = 1;
    n + 2
}]) -> usize {
    a.len()
}
fn main() {
    println!("{}", g([0; 3]));
}
