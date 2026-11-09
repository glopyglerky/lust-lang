// expect: L0631
// rustc E0493: a static initializer drops the copy of the const it projects from.
struct D(u32);
impl Drop for D {
    fn drop(&mut self) {}
}
struct P(D, D);
const NP: P = P(D(1), D(2));
static X: u32 = NP.0.0;
fn main() { println!("{}", X); }
