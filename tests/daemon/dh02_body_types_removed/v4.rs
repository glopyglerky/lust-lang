use std::fmt::Debug;

#[derive(Debug)]
struct Top(i64);

fn a() -> i64 {
    struct P(i64); struct R(i64); struct S(i64); struct T(i64); struct U(i64); struct V(i64);
    P(1).0 + R(2).0 + S(3).0 + T(4).0 + U(5).0 + V(6).0
}
fn b() -> i64 {
    0
}

fn main() {
    println!("{:?} {}", Top(1), a() + b());
}
