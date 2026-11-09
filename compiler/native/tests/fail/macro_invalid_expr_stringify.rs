// expect: L0806
macro_rules! m { ($e:expr) => { stringify!($e) } }
fn main() { let _s: &str = m!((1 2, 3)); }
