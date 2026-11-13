// panic: assertion `left == right` failed
// An `ExactSizeIterator` impl whose `size_hint` is not exact:
// std's default `len` asserts the hint's upper bound equals its
// lower bound, inside core.
struct Count {
    n: usize,
}

impl Iterator for Count {
    type Item = i32;
    fn next(&mut self) -> Option<i32> {
        if self.n > 0 {
            self.n -= 1;
            Some(1)
        } else {
            None
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(99))
    }
}

impl ExactSizeIterator for Count {}

fn main() {
    let c = Count { n: 3 };
    println!("{}", c.len());
}
