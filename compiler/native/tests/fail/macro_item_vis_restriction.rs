// expect: L0806
// rustc: incorrect visibility restriction
// `pub(a)` before an item is no visibility (it would be `pub(in a)`).
macro_rules! m { ($i:item) => { () }; }
fn main() { m!(pub(a) fn h() {}); }
