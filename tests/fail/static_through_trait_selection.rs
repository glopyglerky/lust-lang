// expect: L0306
// rustc E0597: `X: Read` with `trait Read: 'static` (and the impl's `X:
// 'static`) makes `&n` need `'static` (case from fx-dyn-review).
trait Read: 'static { fn read(&self) -> u32; } struct Borrowed<X> { x: X } impl<X: 'static> Read for Borrowed<X> { fn read(&self) -> u32 { 4 } }
fn call<X: Read>(x: &X) -> u32 { x.read() } fn main() { let n = 7u32; let b = Borrowed { x: &n }; println!("{}", call(&b)); }
