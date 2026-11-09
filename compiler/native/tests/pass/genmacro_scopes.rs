// Macro-expanded macro_rules! that rustc accepts: no outer binding of the
// name, a later written definition shadows it, a call inside the same
// expansion, two definitions in one expansion, and a call before it.
macro_rules! d { () => { macro_rules! mym { () => { 1 } } } }
d!();
fn first() -> i32 { mym!() }
macro_rules! mym { () => { 2 } }
fn second() -> i32 { mym!() }
macro_rules! e {
    ($d:tt) => {
        macro_rules! matches { ($d a:expr, $d b:expr) => { "inner" } }
        fn inside() -> &'static str { matches!(1, 2) }
        macro_rules! k { () => { 3 } }
        macro_rules! k { () => { 4 } }
    };
}
fn before() -> bool { matches!(Some(1), Some(_)) }
e!($);
fn main() {
    println!("{} {} {} {} {}", first(), second(), inside(), k!(), before());
}
