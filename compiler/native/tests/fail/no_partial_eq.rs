// expect: L0112
struct P {
    x: i64,
}

fn main() {
    let a = P { x: 1 };
    let b = P { x: 1 };
    println!("{}", a == b);
}
