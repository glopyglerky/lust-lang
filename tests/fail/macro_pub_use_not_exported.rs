// expect: L0510
mod m { macro_rules! k { () => { 1u32 }; } pub use k; }
fn main() { println!("{}", m::k!()); }
