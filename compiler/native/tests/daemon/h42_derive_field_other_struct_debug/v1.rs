#[derive(Debug)]
struct A { x: u8 }
#[derive(Debug)]
struct B { y: u16 }
#[derive(Debug)]
struct S { f: A }
fn main() { println!("{:?}", S { f: A { x: 1 } }); }
