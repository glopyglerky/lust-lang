// expect: L0806
// rustc: expected expression, found `+`
// `stmt` begins on any token, so its parse runs on `+` and fails.
macro_rules! m { ($s:stmt) => { () }; (+) => { () }; }
fn main() { m!(+); }
