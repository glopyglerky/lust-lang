// expect: L0806
// rustc: no rules expected `(`
// A path segment takes `<..>` or `(..)` arguments, not both.
macro_rules! m { ($t:ty , x) => { () }; }
fn main() { m!(Vec<u8> (a) , x); }
