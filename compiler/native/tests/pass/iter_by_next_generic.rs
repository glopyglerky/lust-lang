// Adapters over a user iterator whose `Iterator` impl is generic
// (`impl<T: Copy> Iterator for Gen<T>`): the expansion calls the
// impl's `next`, `size_hint`, and `next_back` instances for the
// concrete type, as with a non-generic impl (iter_by_next.rs).
struct Gen<T> {
    v: Vec<T>,
    i: usize,
    j: usize,
}
impl<T: Copy> Iterator for Gen<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        if self.i >= self.j {
            return None;
        }
        let x = self.v[self.i];
        self.i += 1;
        Some(x)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let left = self.j - self.i;
        (left, Some(left))
    }
}
impl<T: Copy> DoubleEndedIterator for Gen<T> {
    fn next_back(&mut self) -> Option<T> {
        if self.i >= self.j {
            return None;
        }
        self.j -= 1;
        Some(self.v[self.j])
    }
}

fn main() {
    let g = Gen { v: vec![1u32, 2, 3, 4], i: 0, j: 4 };
    let v: Vec<u32> = g.map(|x| x * 10).collect();
    println!("{:?} {}", v, v.capacity());
    let h = Gen { v: vec![5u32, 6, 7, 8], i: 0, j: 4 };
    let r: Vec<u32> = h.rev().collect();
    println!("{:?}", r);
    let s = Gen { v: vec!["a", "bb"], i: 0, j: 2 };
    println!("{}", s.map(|w| w.len()).sum::<usize>());
}
