use std::fmt::Debug;
fn count() -> u32 { static N: u32 = 3; N }
fn pick<T: Copy + Debug>(x: T) -> T { x }
fn f(x: u32, y: u8) -> u32  { x + 1 }
fn user(v: u32) -> u32 { pick(f(v)) }
fn other() -> String { format!("{:?}", pick(f(2))) }
fn main() { println!("{} {} {}", user(count()), other(), f(1)); }
