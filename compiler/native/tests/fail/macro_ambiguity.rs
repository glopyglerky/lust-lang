// expect: L0805
macro_rules! m { ($($i:ident)* $j:ident) => { () } }
fn main() { m!(x); }
