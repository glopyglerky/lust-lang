// expect: L0507
// rustc E0659: each call of `d` defines a different `m`.
macro_rules! d { () => { macro_rules! m { () => { 1 } } } }
d!();
d!();
fn main() { println!("{}", m!()); }
