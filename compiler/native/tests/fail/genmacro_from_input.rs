// expect: L0507
// rustc E0659: a definition passed through a macro's input is macro-expanded.
macro_rules! id { ($($t:tt)*) => { $($t)* } }
id! { macro_rules! vec { ($($t:tt)*) => { 0 } } }
fn main() { let v = vec![1, 2]; println!("{}", v); }
