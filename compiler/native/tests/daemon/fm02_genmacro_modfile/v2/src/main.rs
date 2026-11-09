macro_rules! d { ($d:tt) => { macro_rules! matches { ($d($d t:tt)*) => { true } } } }
d!($);
mod child;
fn main() { child::f(); println!("{}", util::helper()); }
