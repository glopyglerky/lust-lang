macro_rules! d { ($d:tt) => { macro_rules! matches { ($d($d t:tt)*) => { true } } fn inside() -> bool { matches!(1, 2) } } }
d!($);
fn main() { println!("{}", inside()); }
