// A trait with a lifetime param: `impl Tr<'static>` fixes it, so a call through a bound takes `'static` data.
trait Mark<'a> {}
struct S;
impl Mark<'static> for S {}
fn need<'a, T: Mark<'a>>(_x: &'a i32, _t: T) {}
static X: i32 = 1;
fn main() { need(&X, S); println!("ok"); }
