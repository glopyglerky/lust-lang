struct S;
trait A { fn f(&self) -> u8 { 1 } }
trait B { fn f(&self) -> u8 { 2 } }
impl A for S {}
impl B for S {}
fn main() { println!("{} {} {}", A::f(&S), <S as B>::f(&S), B::f(&S)); }
