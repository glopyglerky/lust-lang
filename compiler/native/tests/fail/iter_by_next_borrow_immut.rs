// expect: L0308
// rustc E0596: `find` takes `&mut self`, and `n` is not `mut`.
struct N(u32);
impl Iterator for N {
    type Item = u32;
    fn next(&mut self) -> Option<u32> { None }
}
fn main() {
    let n = N(0);
    println!("{:?}", n.find(|x| *x == 2));
}
