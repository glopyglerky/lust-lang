#[derive(Clone, Copy, PartialEq)]
struct P {
    x: i64,
}

fn twice(p: P) -> (P, P) {
    (p, p)
}

fn main() {
    let p = P { x: 2 };
    let (a, b) = twice(p);
    println!("{:?} {}", a, a == b);
}
