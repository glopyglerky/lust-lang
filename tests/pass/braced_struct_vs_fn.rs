struct S {
    x: i64,
}
fn S() -> i64 {
    7
}
fn main() {
    let s = S { x: 3 };
    println!("{} {}", S(), s.x);
}
