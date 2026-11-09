struct S {
    x: i64,
}
const S: i64 = 5;
fn main() {
    let s = S { x: 3 };
    println!("{} {}", S, s.x);
}
