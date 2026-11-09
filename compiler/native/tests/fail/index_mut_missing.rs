// expect: L0309
// rustc E0594: assigning through an `Index` place needs `IndexMut`.
use std::ops::Index;
struct M(Vec<i32>);
impl Index<usize> for M {
    type Output = i32;
    fn index(&self, i: usize) -> &i32 {
        &self.0[i]
    }
}
fn main() {
    let mut m = M(vec![1]);
    m[0] = 5;
    println!("{}", m[0]);
}
