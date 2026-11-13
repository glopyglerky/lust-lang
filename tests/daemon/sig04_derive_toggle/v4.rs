#[derive(Clone, Copy, Debug, PartialEq, Default)]
struct P {
    x: i64,
}

fn twice(p: P) -> (P, P) {
    (p, p)
}

fn main() {
    let p = P::default();
    let (a, b) = twice(p);
    println!("{:?} {}", a, a == b);
}
