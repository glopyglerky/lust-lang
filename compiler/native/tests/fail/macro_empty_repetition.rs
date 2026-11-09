// expect: L0807
macro_rules! m { ($($v:vis)*) => { 1 } }
fn main() { let _ = m!(); }
