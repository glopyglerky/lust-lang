// The impl's `X: 'static` holds for a `&'static` global and an owned value.
trait Read { fn read(&self) -> u32; }
struct Borrowed<X> { x: X }
impl<X: 'static> Read for Borrowed<X> { fn read(&self) -> u32 { 4 } }
fn call<X: Read>(x: &X) -> u32 { x.read() }
fn main() { static N: u32 = 7; let b = Borrowed { x: &N }; let c = Borrowed { x: 5u32 }; println!("{} {}", call(&b), call(&c)); }
