// expect: L0110
// rustc E0053: `size_hint` returns `(usize, Option<usize>)`.
struct C(u32);
impl Iterator for C {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        None
    }
    fn size_hint(&self) -> (usize, usize) {
        (0, 0)
    }
}
fn main() {
    println!("{}", C(0).count());
}
