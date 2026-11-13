// expect: L0733
use std::thread;
fn sc(s: &thread::Scope) -> u8 { s.spawn(|| 2u8).join().unwrap() }
fn main() { println!("{}", thread::scope(sc)); }
