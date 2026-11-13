// expect: L0507
// rustc E0659: a macro-expanded macro_rules! may not shadow an earlier
// written one, nor one from another call of the same macro.
macro_rules! mym { () => { 2 }; }
macro_rules! d { () => { macro_rules! mym { () => { 1 }; } }; }
d!();
fn main() { println!("{}", mym!()); }
