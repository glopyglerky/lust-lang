macro_rules! mym { ($($t:tt)*) => { false } }
macro_rules! d { ($d:tt) => { macro_rules! mym { ($d($d t:tt)*) => { true } } } }
d!($);
fn main() { let x = Some(1); println!("{}", mym!(x)); }
