macro_rules! d { ($d:tt) => { macro_rules! matches { ($d($d t:tt)*) => { true } } } }
d!($);
fn main() { let x = Some(1); println!("{}", matches!(x, Some(9))); }
