// expect: L0806
// An empty piece between commas does not parse; the recognizer
// must not read `f(1,,2)` as a call with three arguments.
macro_rules! m { ($e:expr) => { "expr" }; ($($t:tt)*) => { "tts" }; }
fn main() { println!("{}", m!(f(1,,2))); }
