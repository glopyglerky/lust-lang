// expect: L0110
// rustc E0271: `Extend<u8>::extend` takes `IntoIterator<Item = u8>`, not `u16`.
struct Bag(Vec<u8>);
impl Extend<u8> for Bag {
    fn extend<I: IntoIterator<Item = u16>>(&mut self, iter: I) {
        for x in iter {
            self.0.push(x as u8);
        }
    }
}
fn main() {
    println!("{}", Bag(vec![]).0.len());
}
