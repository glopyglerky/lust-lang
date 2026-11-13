macro_rules! d { ($d:tt) => { macro_rules! mym { ($d($d t:tt)*) => { 7 } } } }
d!($);
pub fn f() { println!("{}", mym!()); }
