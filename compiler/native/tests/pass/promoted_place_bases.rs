// A constant used as a place in a shared context is promoted to
// `'static`: the base of an index, a field, or an auto-borrowed receiver.
use std::collections::HashMap;
struct I {
    index: HashMap<char, Vec<i32>>,
}
impl I {
    fn f(&self, c: char) -> usize {
        let name_starts: &[i32] = self.index.get(&c).map_or(&[][..], |v| v.as_slice());
        name_starts.len()
    }
}
fn slice() -> &'static [i32] {
    &[1, 2][..]
}
fn at(i: usize) -> &'static i32 {
    &[3, 4][i]
}
fn field() -> &'static i32 {
    &(5, 6).1
}
fn strings() -> &'static [String] {
    [].as_slice()
}
fn main() {
    let i = I { index: HashMap::new() };
    println!("{} {:?} {} {} {}", i.f('a'), slice(), at(1), field(), strings().len());
}
