// expect: L0705
// Valid Rust: `H<X>` is covariant, so `'static` shortens. lust computes no variance for user Adts and asks for the same object lifetime.
trait T { fn read(&self) -> u32; }
struct S { n: u32 }
impl T for S { fn read(&self) -> u32 { self.n } }
struct H<X> { x: X }
fn main() { let s = S { n: 8 }; let x: H<&(dyn T + 'static)> = H { x: &s }; let y: H<&dyn T> = x; println!("{}", y.x.read()); }
