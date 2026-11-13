// expect: L0510
// rustc E0624: a private method of a body impl is private to the fn's module.
mod m {
    pub struct S;
    pub fn f() { impl S { fn hidden(&self) -> u8 { 7 } } }
}
fn main() { m::f(); println!("{}", m::S.hidden()); }
