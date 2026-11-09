// `ExactSizeIterator::len` on the adapters std marks exact, on a
// user iterator with an `ExactSizeIterator` impl (through its
// `size_hint`), and under a generic bound.
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
        (self.n, Some(self.n))
    }
}

impl ExactSizeIterator for Count {}

fn bound_len<I: ExactSizeIterator>(i: I) -> usize {
    i.len()
}

fn main() {
    let v = [1, 2, 3, 4];
    let mut it = v.iter().map(|x| x + 1).enumerate();
    it.next();
    println!("{}", it.len());
    println!("{}", v.iter().zip([9, 8].iter()).len());
    println!("{}", v.iter().skip(3).take(5).len());
    println!("{}", (0..10).step_by(3).len());
    println!("{}", v.iter().copied().rev().len());

    let mut c = Count { n: 3 };
    println!("{}", c.len());
    c.next();
    println!("{}", c.len());
    println!("{}", c.map(|x| x * 2).len());

    println!("{}", bound_len([5, 6].iter()));
    println!("{}", bound_len(Count { n: 2 }));
}
