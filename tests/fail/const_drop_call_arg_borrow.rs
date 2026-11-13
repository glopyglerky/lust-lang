// expect: L0631
// rustc E0493: a call argument borrow is not extending; the temporary drops.
struct D(u32);
impl Drop for D {
    fn drop(&mut self) {}
}
const fn id(x: &D) -> &D { x }
const X: &D = id(&D(1));
fn main() { println!("{}", X.0); }
