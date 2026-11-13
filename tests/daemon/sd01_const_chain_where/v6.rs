use std::fmt::Debug;
fn show<T: Debug>(t: T) -> String { format!("{:?}", t) }
fn f() -> u32  { 5 }
const fn b() -> fn() -> u32 { f }
const fn a() -> fn() -> u32 { b() }
static P: fn() -> u32 = a();
fn other(x: u32) -> u32 where String: Copy { x + 1 }
fn main() { let p = P; println!("{} {}", show(p()), other(2)); }
