// expect: L0110
// rustc E0308: `map` returns `Map<I, F>`, not `I`.
fn f<I: Iterator<Item = i32>>(it: I) -> I { it.map(|x| x) }
fn main() { f(0..3); }
