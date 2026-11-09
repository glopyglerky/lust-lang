// A user `Deref<Target = [T]>` indexed where `T` is still a literal's
// type variable: the step's target type is zonked (lust crash R32,
// ts-rust checker/types.rs `SharedList::slice`).
struct L<T> { v: Vec<T> }
impl<T> std::ops::Deref for L<T> { type Target = [T]; fn deref(&self) -> &[T] { &self.v } }
impl<T: Copy> L<T> {
    fn first_two(&self, range: std::ops::Range<usize>) -> Vec<T> { self[range].to_vec() }
}
fn main() {
    let l = L { v: vec![1, 2, 3, 4] };
    println!("{:?} {}", l.first_two(1..3), l[0]);
    let m = L { v: vec![1.5, 2.5] };
    let x = &m[..1];
    println!("{:?} {} {:?}", x, m.len(), &m[1..]);
    let t = L { v: vec![(1, -2), (3, 4)] };
    println!("{:?}", t[1]);
}
