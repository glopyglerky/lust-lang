// feature: macros.user
// form: macro_rules! with expr, ident and repetition, as in ts-rust flag macros
macro_rules! sum { ($($e:expr),* $(,)?) => { 0 $(+ $e)* }; }
macro_rules! make_getter { ($name:ident, $v:expr) => { fn $name() -> i32 { $v } }; }
make_getter!(answer, 42);
fn main() { println!("{} {}", sum!(1, 2, 3), answer()); }
