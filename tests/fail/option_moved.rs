// expect: L0301
fn main() {
    let s = Some(String::from("x"));
    let a = s.unwrap();
    let b = s.unwrap();
    println!("{} {}", a, b);
}
