// A private method of a body impl is visible in the fn's module.
mod m {
    pub struct S;
    pub fn f() -> u8 { impl S { pub fn shown(&self) -> u8 { 7 } fn hid(&self) -> u8 { 1 } } S.hid() }
}
fn main() { println!("{} {}", m::f(), m::S.shown()); }
