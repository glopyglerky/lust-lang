// expect: L0806
macro_rules! m { ($e:expr ;) => { 1 }; (1 +) => { 2 } }
fn main() { let _ = m!(1 +); }
