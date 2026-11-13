// expect: L0306
// ARCH 2.8, rustc E0597: `child` returns a `&dyn Node` tied to `n`.
trait Node { fn child(&self) -> &dyn Node; fn v(&self) -> u32; }
impl dyn Node { fn depth(&self) -> u32 { self.v() } }
struct Leaf;
impl Node for Leaf { fn child(&self) -> &dyn Node { self } fn v(&self) -> u32 { 1 } }
fn main() { let n = Leaf; let d: &dyn Node = &n; println!("{}", d.child().depth()); }
