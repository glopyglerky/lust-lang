// expect: L0615
// rustc E0119: alloc has `impl<I: Iterator> Iterator for Box<I>`.
struct Local;
impl Iterator for Local { type Item = i32; fn next(&mut self) -> Option<i32> { None } }
impl Iterator for Box<Local> { type Item = i32; fn next(&mut self) -> Option<i32> { Some(1) } }
fn main() { println!("{:?}", Box::new(Local).next()); }
