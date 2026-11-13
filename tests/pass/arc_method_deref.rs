use std::sync::Arc;
struct S { v: Vec<u32> }
impl S { fn sum(&self) -> u32 { self.v.iter().sum() } fn first(&self) -> Option<&u32> { self.v.first() } }
fn main() {
    let a = Arc::new(S { v: vec![4, 5] });
    println!("{} {:?} {}", a.sum(), a.first(), a.v.len());
    let r: &S = &a;
    println!("{}", r.sum());
    let t = Arc::new([1, 2, 3]);
    println!("{} {}", t[1], t.len());
}
