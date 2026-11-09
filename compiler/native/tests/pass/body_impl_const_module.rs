// A body impl's const initializer names an item of the fn's module.
mod m {
    const BASE: u32 = 40;
    pub struct S;
    pub fn f() -> u32 { impl S { pub const V: u32 = BASE + 2; } S::V }
}
fn main() { println!("{} {}", m::f(), m::S::V); }
