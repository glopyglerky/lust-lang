// ARCH 2.5: an associated equality in a bound normalizes the
// projection inside a generic body, with no concrete impl known.
trait Iter {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}
struct Count {
    n: u8,
}
impl Iter for Count {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        if self.n == 0 {
            None
        } else {
            self.n -= 1;
            Some(self.n)
        }
    }
}
// Item = u8 is part of the signature. The sum has type u8 only
// through that equality.
fn sum<I: Iter<Item = u8>>(mut it: I) -> u8 {
    let mut total: u8 = 0;
    while let Some(x) = it.next() {
        total += x;
    }
    total
}
fn main() {
    println!("{}", sum(Count { n: 4 }));
}
