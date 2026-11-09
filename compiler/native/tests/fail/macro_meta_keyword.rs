// expect: L0806
// rustc: expected identifier, found keyword `pub`
// `meta` can begin on any identifier, keywords included, and its path
// parse then fails: the call fails before the `tt` arm is tried.
macro_rules! m { ($m:meta) => { () }; ($t:tt) => { () }; }
fn main() { m!(pub); }
