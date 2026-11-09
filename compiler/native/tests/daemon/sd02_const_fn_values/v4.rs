use std::fmt::Debug;
struct H { f: fn(u32) -> u32 }
fn show<T: Debug>(t: T) -> String { format!("{:?}", t) }
fn dbl(x: u64) -> u32  { x * 2 }
fn inc(x: u32) -> u32  { x + 1 }
const HH: H = H { f: dbl };
static T: [fn(u32) -> u32; 2] = [inc, inc];
fn main() { println!("{} {} {}", show((HH.f)(3)), (T[0])(4), inc(1)); }
