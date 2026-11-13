#[derive(Clone)]
struct S {
    a: u64,
    n: i64,
}
fn main() {
    let s = S { a: 7, n: 3 };
    println!("{}", s.n);
    let t = s.clone();
    println!("{} {}", t.a, t.n);
}
