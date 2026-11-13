// expect: L0110
struct S {
    x: i32,
    y: i32,
}

fn main() {
    let s = S { x: 1, .. };
    println!("{}", s.x);
}
