// expect: L0107
struct S {
    x: i32,
    y: i32,
}

fn main() {
    let mut a = 0;
    S { x: a } = S { x: 1, y: 2 };
    println!("{}", a);
}
