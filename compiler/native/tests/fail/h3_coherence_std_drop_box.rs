// expect: L0615
// rustc E0119: alloc has `impl<T> Drop for Box<T>`.
struct Local;
impl Drop for Box<Local> { fn drop(&mut self) { println!("drop box"); } }
fn main() { let _b = Box::new(Local); println!("ok"); }
