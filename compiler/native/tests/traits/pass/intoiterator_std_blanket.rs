// ARCH 2.6: the std blanket `impl<I: Iterator> IntoIterator for I`
// makes every Iterator usable in a `for` loop with no extra impl.
// A local iterator type gets IntoIterator from the blanket alone.
struct Count {
    n: u8,
}
impl Iterator for Count {
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
fn main() {
    let mut total = 0u8;
    for x in (Count { n: 4 }) {
        total += x;
    }
    println!("{total}");
}
