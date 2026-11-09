use std::fmt::Debug;
mod m { pub fn f() -> u32 { 7 } }
use m::f as k;
struct S;
impl S { fn g(&self) -> impl Debug { k() } fn plain(&self, x: u32) -> u32  { x } }
fn need<T: Send>(t: T) -> T { t }
fn main() { let s = S; println!("{:?} {}", need(s.g()), s.plain(1u8)); }
