// expect: L0202
// rustc E0106: a lifetime written inside a by-value `self` type does not count.
struct W<'a> {
    r: &'a u8,
}
impl<'a> W<'a> {
    fn get(self: W<'a>) -> &u8 {
        self.r
    }
}
fn main() {}
