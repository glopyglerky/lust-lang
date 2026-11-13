mod child;
macro_rules! d { ($d:tt) => { macro_rules! mym { ($d($d t:tt)*) => { 5 } } } }
d!($);
fn main() { child::f(); println!("{} {}", util::helper(), mym!()); }
