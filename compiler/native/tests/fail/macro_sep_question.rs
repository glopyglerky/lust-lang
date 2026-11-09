// expect: L0807
macro_rules! m { ($($i:ident),?) => { 1 } }
fn main() { let _ = m!(a); }
