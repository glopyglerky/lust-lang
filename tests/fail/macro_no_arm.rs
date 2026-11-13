// expect: L0806
macro_rules! m { (yes) => { 1 } }
fn main() { let _ = m!(no); }
