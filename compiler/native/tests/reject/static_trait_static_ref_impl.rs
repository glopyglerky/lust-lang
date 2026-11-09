// expect: L0612
// Valid Rust: the header reference is `&'static`. lust drops the lifetime of a non-dyn reference, so it cannot see it.
trait T: 'static { fn f(&self) -> u32 { 3 } }
impl T for &'static u32 {}
static N: u32 = 4;
fn main() { let r: &'static u32 = &N; println!("{}", r.f()); }
