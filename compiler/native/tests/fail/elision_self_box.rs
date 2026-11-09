// expect: L0202
// rustc E0106: `self: Box<Self>` has no reference to `Self`.
struct W<'a> {
    r: &'a u8,
}
impl<'a> W<'a> {
    fn get(self: Box<Self>) -> &u8 {
        self.r
    }
}
fn main() {}
