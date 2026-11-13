// expect: L0802
fn main() { let _ = m!(); }
macro_rules! m { () => { 1 } }
