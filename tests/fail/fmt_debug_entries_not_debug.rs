// expect: L0112
// rustc E0277: the items of `entries` must be Debug.
use std::fmt;
struct NoDbg;
struct W;
impl fmt::Debug for W {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(vec![NoDbg]).finish()
    }
}
fn main() {
    println!("{:?}", W);
}
