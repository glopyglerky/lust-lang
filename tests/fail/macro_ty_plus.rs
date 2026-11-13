// expect: L0806
// rustc: expected a path on the left-hand side of `+`
// rustc reads bounds after any type, and rejects them after `!`.
macro_rules! m { ($t:ty >) => { () }; (! $($t:tt)*) => { () }; }
fn main() { m!(! + 1); }
