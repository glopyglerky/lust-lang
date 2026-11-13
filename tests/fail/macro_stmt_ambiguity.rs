// expect: L0805
// rustc: local ambiguity when calling macro `m`: multiple parsing options: built-in NTs stmt ('s') or 1 other option.
// `stmt` can begin on any token, so it is a candidate next to the `;`.
macro_rules! m { ($($s:stmt)? ; x) => { () }; }
fn main() { m!(; x); }
