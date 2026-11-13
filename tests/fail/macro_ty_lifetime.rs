// expect: L0806
// rustc: expected type, found lifetime
// `ty` can begin on a lifetime (a bound), but a lone lifetime is no type.
macro_rules! m { ($t:ty ; x) => { () }; ($l:lifetime b) => { () }; }
fn main() { m!('static b); }
