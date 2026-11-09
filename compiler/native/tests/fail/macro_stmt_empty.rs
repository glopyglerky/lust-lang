// expect: L0806
// rustc: no rules expected `(`
// A leading `;` is the whole (empty) statement.
macro_rules! m { ($s:stmt ,) => { () }; }
fn main() { m!(; (a) ,); }
