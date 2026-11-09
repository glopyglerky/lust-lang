// expect: L0612
// rustc E0271: `DebugMap::entries` takes `(K, V)` items.
use std::fmt;
struct W;
impl fmt::Debug for W {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(vec![1, 2]).finish()
    }
}
fn main() {
    println!("{:?}", W);
}
