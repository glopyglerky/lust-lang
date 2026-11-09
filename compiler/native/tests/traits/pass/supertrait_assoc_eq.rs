// ARCH 2.5: an associated equality written on a supertrait bound
// reaches a body through the elaborated param env. The body is
// bounded only by ByteIter, yet Item must normalize to u8.
trait Iter {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}
trait ByteIter: Iter<Item = u8> {}
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
impl ByteIter for Count {}
fn sum<I: ByteIter>(mut it: I) -> u8 {
    let mut total: u8 = 0;
    while let Some(x) = it.next() {
        total += x;
    }
    total
}
fn main() {
    println!("{}", sum(Count { n: 3 }));
}
