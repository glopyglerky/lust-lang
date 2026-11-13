// expect: L0306
// rustc E0597: unsizing `B<'_>` to `dyn Tk` needs `B<'_>: Tk`, and the only impl is for
// `B<'static>`; `b` borrows a local.
trait Tk { fn tk(&self) -> u32; }
struct B<'b>(&'b u32);
impl Tk for B<'static> { fn tk(&self) -> u32 { *self.0 } }
fn show(d: &dyn Tk) -> u32 { d.tk() }
fn main() { let m = 4u32; let b = B(&m); println!("{}", show(&b)); }
