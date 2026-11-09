// expect: L0305
// rustc E0507: a by-value `self` of a non-Copy type behind a reference.
struct W(u32, String);
impl W {
    const fn take(self) -> u32 {
        let W(v, s) = self;
        std::mem::forget(s);
        v
    }
}
const R: &W = &W(1, String::new());
const X: u32 = R.take();
fn main() {
    println!("{}", X);
}
