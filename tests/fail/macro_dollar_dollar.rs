// expect: L0807
// rustc: meta-variable expressions are unstable
macro_rules! m { () => { $$x }; (a) => { 1 }; }
fn main() { let _ = m!(a); }
