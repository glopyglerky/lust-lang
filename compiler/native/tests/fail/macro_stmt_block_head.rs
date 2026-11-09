// expect: L0806
// rustc: no rules expected `-`
// A statement that starts with a block ends after it: `{ 1 } - 1` is
// two statements.
macro_rules! m { ($s:stmt , x) => { () }; }
fn main() { m!({ 1 } - 1 , x); }
