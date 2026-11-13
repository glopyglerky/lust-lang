// expect: L0304
// rustc E0499: the DebugStruct holds `&mut Formatter` until finish.
use std::fmt;
struct X;
impl fmt::Debug for X {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_struct("X");
        f.write_str("y")?;
        d.finish()
    }
}
fn main() {
    println!("{:?}", X);
}
