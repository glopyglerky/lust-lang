use std::fmt::Debug;
fn show<T: Debug + Copy>(t: T) -> String { format!("{:?}", t) }
fn twice<T>(t: T) -> (T, T) where T: Clone { (t.clone(), t) }
fn plain(x: u32) -> u32  { x }
fn main() { let s = String::from("a"); println!("{} {:?} {}", show(s.clone()), twice(s), plain(3)); }
