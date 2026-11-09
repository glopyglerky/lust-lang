// expect: L0705
// Valid Rust: lust does not check `&'static` references to `dyn` values in impl headers.
trait T { fn read(&self) -> u32; }
struct S { n: u32 }
impl T for S { fn read(&self) -> u32 { self.n } }
trait Tr { fn tr(&self) -> u32; }
impl Tr for &'static dyn T { fn tr(&self) -> u32 { self.read() } }
static S0: S = S { n: 4 };
fn main() { let d: &'static dyn T = &S0; println!("{}", d.tr()); }
