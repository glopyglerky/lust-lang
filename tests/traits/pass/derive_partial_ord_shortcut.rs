// ARCH 2.2 derives: rustc's derived partial_cmp on a non-generic type that
// also derives Ord is Some(self.cmp(other)), so a field with a hand-written
// Ord decides `<` through that cmp. A generic type keeps field partial_cmp.
use std::cmp::Ordering;
#[derive(PartialEq, Eq, PartialOrd, Debug)]
struct Rev(u8);
impl Ord for Rev {
    fn cmp(&self, o: &Rev) -> Ordering {
        o.0.cmp(&self.0)
    }
}
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Plain(Rev, u8);
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Gen<T>(T, u8);
fn main() {
    println!("{} {:?}", Plain(Rev(1), 0) < Plain(Rev(2), 0), Plain(Rev(1), 0).partial_cmp(&Plain(Rev(2), 0)));
    println!("{} {:?}", Gen(Rev(1), 0) < Gen(Rev(2), 0), Gen(Rev(1), 0).partial_cmp(&Gen(Rev(2), 0)));
}
