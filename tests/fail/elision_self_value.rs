// expect: L0202
// rustc E0106: a by-value `self` has no input lifetime for elision.
struct W<'a> {
    r: &'a u8,
}
impl<'a> W<'a> {
    fn get(self) -> &u8 {
        self.r
    }
}
fn main() {}
