use std::fmt::Debug;
fn pick<T: Copy + Debug>(x: T) -> T { x }
fn f() -> i8 { 7 }
fn user() -> String { format!("{:?}", pick(f())) }
fn main() { println!("{}", user()); }
