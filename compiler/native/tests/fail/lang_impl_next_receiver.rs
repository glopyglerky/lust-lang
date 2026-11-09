// expect: L0110
// rustc E0053: `next` takes `&mut self`.
struct C(u32);
impl Iterator for C {
    type Item = u32;
    fn next(&self) -> Option<u32> {
        None
    }
}
fn main() {
    println!("{}", C(0).count());
}
