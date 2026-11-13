// User `DoubleEndedIterator` and `Extend` impls (W-STD-B): `next_back`
// called directly and by `rev`, `Self::Item` from the supertrait, and a
// user `extend` called with Vec arguments.
struct N(u32, u32);
impl Iterator for N {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        if self.0 < self.1 {
            self.0 += 1;
            Some(self.0)
        } else {
            None
        }
    }
}
impl DoubleEndedIterator for N {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.0 < self.1 {
            self.1 -= 1;
            Some(self.1 + 1)
        } else {
            None
        }
    }
}
struct Bag(Vec<u8>);
impl Extend<u8> for Bag {
    fn extend<I: IntoIterator<Item = u8>>(&mut self, iter: I) {
        for x in iter {
            self.0.push(x);
        }
    }
}
struct Log(String);
impl<'a> Extend<&'a str> for Log {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = &'a str>,
    {
        for s in iter {
            self.0.push_str(s);
            self.0.push(';');
        }
    }
}
fn main() {
    let mut n = N(0, 5);
    println!("{:?} {:?} {:?}", n.next_back(), n.next(), n.next_back());
    let v: Vec<u32> = N(0, 4).rev().collect();
    let w: Vec<u32> = N(0, 4).map(|x| x * 10).rev().take(2).collect();
    println!("{v:?} {w:?}");
    let mut b = Bag(vec![]);
    b.extend(vec![1u8, 2]);
    b.extend(Vec::new());
    println!("{:?}", b.0);
    let mut l = Log(String::new());
    l.extend(vec!["a", "bc"]);
    println!("{}", l.0);
}
