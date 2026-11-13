// expect: L0806
// A `..=` with no end does not parse as an expression in rustc;
// the fragment parse fails hard instead of falling to a tt arm.
macro_rules! m { ($e:expr) => { "expr" }; ($($t:tt)*) => { "tts" }; }
fn main() { println!("{}", m!(..=)); }
