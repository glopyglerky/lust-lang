// expect: L0806
// rustc: inclusive range with no end
// rustc reports this error while it tries the first arm (the index
// holds `..=`), so the call fails although the second arm matches.
macro_rules! m { ($e:expr => x) => { () }; ($($t:tt)*) => { () }; }
fn main() { m!({ 1 } [..=]); }
