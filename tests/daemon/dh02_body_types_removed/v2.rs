use std::fmt::Debug;

#[derive(Debug)]
struct Top(i64);

fn b() -> i64 {
    struct Q(i64); struct W(i64); struct X(i64); struct Y(i64); struct Z(i64); struct O(i64);
    Q(10).0 + W(11).0 + X(12).0 + Y(13).0 + Z(14).0 + O(15).0
}

fn main() {
    println!("{:?} {}", Top(1), b());
}
