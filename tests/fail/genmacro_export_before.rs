// expect: L0507
// rustc E0659: a macro-expanded `#[macro_export]` macro is in the crate
// root from the start of the file, ambiguous with the std `vec`.
fn main() { let v = vec![1]; println!("{:?}", v); }
macro_rules! d { ($d:tt) => { #[macro_export] macro_rules! vec { ($d($d t:tt)*) => { 4 } } } }
d!($);
