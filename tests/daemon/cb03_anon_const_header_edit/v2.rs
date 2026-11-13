fn g(a: [u8; {
    if false {
        let n = 1;
        n = 2;
    }
    3
}]) -> usize {
    a.len()
}
fn main() {
    println!("{}", g([0; 3]));
}
