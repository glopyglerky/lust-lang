// expect: L0807
macro_rules! m { ($x:ident) => { $($x)* } }
fn main() { m!(a); }
