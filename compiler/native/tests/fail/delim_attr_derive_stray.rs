// expect: L0001
// rustc: mismatched closing delimiter. The derive list's `]` is missing; a stray `]` follows.
#[derive(Debug, Clone, PartialEq)struct Qq {]
struct Point {
    x: i64,
}

fn main() {
    let p = Point { x: 1 };
    println!("{}", p.x);
}
