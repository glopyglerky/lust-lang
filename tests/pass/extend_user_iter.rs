// `extend` from a user iterator (W-STD-B): Vec, String, HashSet; the
// reserve follows its `size_hint`.
struct Count(u32, u32);
impl Iterator for Count {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        if self.0 < self.1 {
            self.0 += 1;
            Some(self.0)
        } else {
            None
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = (self.1 - self.0) as usize;
        (n, Some(n))
    }
}
fn main() {
    let mut set: std::collections::HashSet<u32> = std::collections::HashSet::new();
    set.extend(Count(0, 5));
    println!("{}", set.len());
    let c = Count(0, 10);
    println!("{:?}", c.size_hint());
    let v: Vec<u32> = Count(0, 10).collect();
    println!("{:?} {}", v, v.capacity());
    let w: Vec<u32> = Count(0, 10).map(|x| x * 2).collect();
    println!("{} {:?}", w.capacity(), Count(2, 5).map(|x| x).size_hint());
    let mut e = vec![1u32];
    e.extend(Count(0, 20));
    println!("{}", e.capacity());
    let s: String = Count(0, 9).map(|d| char::from(b'0' + d as u8)).collect();
    println!("{s} {}", s.capacity());
}
