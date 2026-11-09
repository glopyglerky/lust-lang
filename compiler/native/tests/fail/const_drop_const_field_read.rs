// expect: L0631
// rustc E0493: a const item path is a value; `C.1` reads a field of a fresh copy that is dropped.
struct D(u32);
impl Drop for D {
    fn drop(&mut self) {}
}
const C: (D, u32) = (D(1), 5);
const X: u32 = C.1;
fn main() { println!("{}", X); }
