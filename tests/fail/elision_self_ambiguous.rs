// expect: L0202
// rustc E0106: two references to `Self` in the receiver leave no elision.
struct W<'a> {
    r: &'a u8,
}
impl<'a> W<'a> {
    fn get(self: &Box<&Self>) -> &u8 {
        self.r
    }
}
fn main() {}
