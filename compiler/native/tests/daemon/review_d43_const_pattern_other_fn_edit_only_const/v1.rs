const K: i32 = 1;
fn f(x: i32) -> bool { matches!(x, K) }
fn main() { println!("{} {}", f(1), f(2)); }
