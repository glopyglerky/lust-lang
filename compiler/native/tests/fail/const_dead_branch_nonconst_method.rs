// expect: L0631
// rustc E0015: a non-const call in an untaken branch of a const.
struct W(u32);
impl W {
    fn nc(&self) -> u32 {
        self.0
    }
}
const fn f(w: &W) -> u32 {
    w.0
}
const X: u32 = {
    let w = W(3);
    if false { w.nc() } else { f(&w) }
};
fn main() {
    println!("{}", X);
}
