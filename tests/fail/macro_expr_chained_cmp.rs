// expect: L0806
// rustc: comparison operators cannot be chained
macro_rules! m { ($e:expr ;) => { () }; ($($t:tt)*) => { () }; }
fn main() { m!(a < b > ::c ;); }
