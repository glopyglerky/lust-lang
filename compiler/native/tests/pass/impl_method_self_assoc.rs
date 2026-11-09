// An impl method's signature that names `Self::Assoc` means the impl's
// binding (codegen typed it as `()`).
trait Make {
    type Out;
    fn make(&self) -> Self::Out;
    fn take(&self, x: Self::Out) -> usize;
}

struct A;

impl Make for A {
    type Out = Vec<u8>;
    fn make(&self) -> Self::Out {
        vec![1, 2]
    }
    fn take(&self, x: Self::Out) -> usize {
        x.len()
    }
}

struct Counter {
    n: u8,
}

impl Iterator for Counter {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        if self.n < 3 {
            self.n += 1;
            Some(self.n)
        } else {
            None
        }
    }
}

struct Bag {
    v: Vec<u8>,
}

impl IntoIterator for Bag {
    type Item = u8;
    type IntoIter = std::vec::IntoIter<u8>;
    fn into_iter(self) -> Self::IntoIter {
        self.v.into_iter()
    }
}

fn main() {
    let m = A.make();
    let n = A.take(vec![9]);
    let mut c = Counter { n: 0 };
    let first = c.next();
    let s: u8 = Bag { v: vec![1, 2, 3] }.into_iter().sum();
    println!("{:?} {} {:?} {}", m, n, first, s);
}
