// expect: L0615
// rustc E0751: core has `impl<T: ?Sized> !Clone for &mut T`.
struct Local;
impl Clone for &mut Local { fn clone(&self) -> Self { todo!() } }
fn main() { println!("ok"); }
