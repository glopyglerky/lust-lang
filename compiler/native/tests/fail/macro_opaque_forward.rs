// expect: L0806
macro_rules! inner { (3) => { 3 } }
macro_rules! outer { ($x:expr) => { inner!($x) } }
fn main() { let _ = outer!(3); }
