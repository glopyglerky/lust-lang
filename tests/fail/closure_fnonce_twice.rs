// expect: L0301
fn main() {
    let s = String::from("once");
    let eat = move || s;
    let a = eat();
    let b = eat();
    println!("{} {}", a, b);
}
