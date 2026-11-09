// expect: L0110
// rustc E0049: `extend` has one type parameter.
struct Bag(Vec<u8>);
impl Extend<u8> for Bag {
    fn extend(&mut self, x: u8) {
        self.0.push(x);
    }
}
fn main() {
    println!("{}", Bag(vec![]).0.len());
}
