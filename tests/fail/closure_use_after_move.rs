// expect: L0301
fn main() {
    let s = String::from("gone");
    let c = move || s.len();
    println!("{} {}", c(), s);
}
