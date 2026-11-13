// expect: L9001
struct S {
    x: i32,
}
fn main() {
    let mut s = S { x: 1 };
    let o = std::mem::take(&mut s);
    println!("{}", o.x);
}
