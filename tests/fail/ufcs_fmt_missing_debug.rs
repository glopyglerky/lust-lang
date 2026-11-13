// expect: L0612
// rustc E0277: `Debug::fmt` needs `P: Debug`, which P lacks.
use std::fmt;
struct P;
struct W(P);
impl fmt::Debug for W {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}
fn main() {
    println!("{:?}", W(P));
}
