#[macro_export]
macro_rules! vec { ($($t:tt)*) => { [99] }; }
#[macro_export]
macro_rules! format { ($($t:tt)*) => { String::from("Ufmt") }; }
#[macro_export]
macro_rules! concat { ($($t:tt)*) => { "Ucat" }; }
#[macro_export]
macro_rules! matches { ($e:expr, $p:pat) => { true }; }
#[macro_export]
macro_rules! stringify { ($($t:tt)*) => { "Ustr" }; }
pub fn helper() -> i64 { 0 }
