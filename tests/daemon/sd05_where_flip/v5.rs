use std::fmt::Debug;
fn show<T: Debug>(t: T) -> String { format!("{:?}", t) }
fn twice<T>(t: T) -> (T, T) where T: Copy { (t.clone(), t) }
fn plain(x: u32) -> u32  { x }
fn main() { let s = String::from("a"); println!("{} {:?} {}", show(s.clone()), twice(1u8), plain(3)); }
