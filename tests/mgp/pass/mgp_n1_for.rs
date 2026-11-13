// MGP (ARCH 9.3): the N1 shape. A concrete IntoIterator impl
// for a local type and the bounded bare-param blanket
// `impl<I: Iterator> IntoIterator for I` both match by header.
// The local type is not an Iterator, so the clause filter must
// drop the blanket and select the concrete impl, through a
// `for` loop over a local iterator type.
struct CountDown {
    next: u8,
}
impl Iterator for CountDown {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        if self.next == 0 {
            None
        } else {
            let value = self.next;
            self.next -= 1;
            Some(value)
        }
    }
}
struct Range {
    top: u8,
}
impl IntoIterator for Range {
    type Item = u8;
    type IntoIter = CountDown;
    fn into_iter(self) -> CountDown {
        CountDown { next: self.top }
    }
}
fn main() {
    let mut total = 0u16;
    let mut count = 0u16;
    for x in (Range { top: 4 }) {
        total += x as u16;
        count += 1;
    }
    println!("{total} {count}");
}
