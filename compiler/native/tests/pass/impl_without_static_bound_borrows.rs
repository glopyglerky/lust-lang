// An impl with no `'static` where-clause asks nothing of the borrow.
trait Read { fn read(&self) -> u32; }
struct Borrowed<X> { x: X }
impl<X> Read for Borrowed<X> { fn read(&self) -> u32 { 4 } }
fn call<X: Read>(x: &X) -> u32 { x.read() }
fn main() { let n = 7u32; let b = Borrowed { x: &n }; println!("{}", call(&b)); }
