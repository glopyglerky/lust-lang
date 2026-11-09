// expect: L0631
// rustc E0493: a `let` borrow extends only to its block, which ends inside the const.
struct D(u32);
impl Drop for D {
    fn drop(&mut self) {}
}
const X: &D = { let r = &D(1); r };
fn main() { println!("{}", X.0); }
