// expect: L0510
// rustc E0624: a private method of a body impl in a child module.
mod a {
    pub struct S;
    pub mod b {
        pub fn f() -> u8 { impl super::S { pub(super) fn p(&self) -> u8 { 3 } fn q(&self) -> u8 { 4 } } super::S.q() }
    }
    pub fn g() -> u8 { S.q() }
}
fn main() { println!("{} {}", a::b::f(), a::g()); }
