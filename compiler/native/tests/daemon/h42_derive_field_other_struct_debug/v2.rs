#[derive(Debug)]
struct A { x: u8 }
#[derive(Debug)]
struct B { y: u16 }
#[derive(Debug)]
struct S { f: B }
fn main() { println!("{:?}", S { f: B { y: 1 } }); }
