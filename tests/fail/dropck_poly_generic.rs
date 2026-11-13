// expect: L0730
// rustc E0320 on the generic body alone: mk is never called.
struct Grow<T> { v: T, g: Vec<Grow<Option<T>>> }
fn mk<T>(v: T) -> Grow<T> { Grow { v, g: Vec::new() } }
fn main() { println!("ok"); }
