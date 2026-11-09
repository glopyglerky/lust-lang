// expect: L0705
// Valid Rust: rustc infers `'static` for the object lifetime of `b`. lust treats a body annotation under `&mut` like a signature and asks for the same object lifetime.
trait T { fn read(&self) -> u32; }
struct S { n: u32 }
impl T for S { fn read(&self) -> u32 { self.n } }
static S0: S = S { n: 4 };
fn main() { let mut r: &(dyn T + 'static) = &S0; let a: &mut &(dyn T + 'static) = &mut r; let b: &mut &dyn T = a; println!("{}", b.read()); }
