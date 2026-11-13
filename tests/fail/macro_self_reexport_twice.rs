// expect: L0106
mod m { macro_rules! k { () => { 1u32 }; } pub(crate) use k; pub(crate) use k; pub fn f() -> u32 { k!() } }
fn main() { println!("{}", m::f()); }
