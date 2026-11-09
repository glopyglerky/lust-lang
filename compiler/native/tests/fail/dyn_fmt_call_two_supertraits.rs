// expect: L0720
// rustc E0034: at the `dyn Both` step both supertraits' `fmt` are object candidates, ahead of
// the impl's own trait (in scope only as an extension candidate).
use std::fmt;
trait Both: fmt::Debug + fmt::Display {}
impl Both for u8 {}
struct B<'a>(&'a dyn Both);
impl fmt::Debug for B<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
fn main() {
    println!("{:?}", B(&2));
}
