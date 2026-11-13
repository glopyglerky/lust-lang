// expect: L0705
// Valid Rust: `d` borrows a static, so `&dyn T: 'static` holds. lust keeps no lifetime on `&dyn T` written without `'static`.
trait T { fn read(&self) -> u32; }
struct S { n: u32 }
impl T for S { fn read(&self) -> u32 { self.n } }
static S0: S = S { n: 4 };
fn need<X: 'static>(_x: X) -> u32 { 7 }
fn main() { let d: &dyn T = &S0; println!("{}", need(d)); }
