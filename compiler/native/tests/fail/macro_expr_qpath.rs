// expect: L0806
// rustc: expected `::`, found `<eof>`
// A qualified path needs `::` and a segment after its `>`.
macro_rules! m { ($e:expr) => { () }; ($($t:tt)*) => { () }; }
fn main() { m!(<X>); }
