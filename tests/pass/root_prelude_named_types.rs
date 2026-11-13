// Crate-root items named like std prelude types shadow the prelude in
// the root; modules that do not import them still see std's.
type Result<T> = std::result::Result<T, String>;
struct Vec { x: i32 }
impl Vec { fn new() -> Vec { Vec { x: 7 } } }
#[derive(Debug)]
struct Box(i32);
mod m {
    pub fn f() -> Option<i32> { Some(1) }
    pub fn v() -> Vec<u8> { vec![1, 2] }
}
fn f(x: i32) -> Result<i32> { if x > 0 { Ok(x) } else { Err("neg".to_string()) } }
fn main() {
    println!("{:?} {:?}", f(1), f(-1));
    println!("{} {:?}", Vec::new().x, Box(3));
    println!("{:?} {:?}", m::f(), m::v());
}
