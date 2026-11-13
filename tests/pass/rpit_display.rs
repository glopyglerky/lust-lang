// impl Display values formatted and passed to a generic Display bound (W-CHECK-G3).
use std::fmt::Display;
fn label(n: u8) -> impl Display { format!("n={}", n) }
fn twice<T: Display>(t: T) -> String { format!("{t}{t}") }
fn main() { let l = label(4); println!("{} {}", l, twice(label(1))); }
