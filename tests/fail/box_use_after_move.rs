// expect: L0301
fn main() {
    let b = Box::new("x".to_string());
    let s = *b;
    println!("{} {}", s, b.len());
}
