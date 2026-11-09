// expect: L0121
struct S {
    x: i32,
    y: i32,
}

fn main() {
    let mut a = 0;
    let mut b = 0;
    let base = S { x: 1, y: 2 };
    S { x: a, y: b, ..base } = S { x: 3, y: 4 };
    println!("{} {}", a, b);
}
