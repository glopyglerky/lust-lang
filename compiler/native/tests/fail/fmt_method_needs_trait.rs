// expect: L0104
// rustc E0599: `fmt` on u32 needs Display or Debug in scope; only an
// impl of the trait puts it in scope inside the impl.
use std::fmt;
fn show(x: u32, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    x.fmt(f)
}
struct T(u32);
impl fmt::Display for T {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        show(self.0, f)
    }
}
fn main() {
    println!("{}", T(1));
}
