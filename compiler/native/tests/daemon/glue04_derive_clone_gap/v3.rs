#[derive(Clone)]
struct S {
    a: String,
    n: i64,
}
fn main() {
    let s = S { a: String::from("hello"), n: 3 };
    println!("{}", s.n);
    let t = s.clone();
    println!("{} {}", t.a, t.n);
}
