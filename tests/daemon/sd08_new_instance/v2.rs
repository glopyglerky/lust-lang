use std::fmt::Debug;
fn pick<T: Copy + Debug>(x: T) -> T { x }
fn f() -> u64 { 7 }
fn user() -> String { format!("{:?}", pick(f())) }
fn main() { println!("{}", user()); }
