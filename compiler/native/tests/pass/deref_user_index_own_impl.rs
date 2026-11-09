// Method calls autoderef through user Deref impls, but indexing
// uses the type's own Index impl first (W-STD-C phase 2): w[0] is
// the reversed index, w.len() goes through the Deref.
use std::ops::{Deref, Index};
struct W(Vec<u32>);
impl Deref for W {
    type Target = Vec<u32>;
    fn deref(&self) -> &Vec<u32> {
        &self.0
    }
}
impl Index<usize> for W {
    type Output = u32;
    fn index(&self, i: usize) -> &u32 {
        &self.0[self.0.len() - 1 - i]
    }
}
fn main() {
    let w = W(vec![1, 2, 3]);
    println!("{} {}", w[0], w.len());
}
