// A projection in an impl header's trait args (`impl<A: Array>
// Sink<A::Item> for SmallVec<A>`, the smallvec Extend/From shape)
// matches after normalization, as rustc's match_impl does.
trait Array {
    type Item;
}
impl<T> Array for [T; 4] {
    type Item = T;
}
impl<T> Array for [T; 2] {
    type Item = T;
}
struct SmallVec<A: Array> {
    data: Vec<A::Item>,
}
trait Sink<X> {
    fn put(&mut self, x: X);
    fn count(&self) -> usize;
}
impl<A: Array> Sink<A::Item> for SmallVec<A> {
    fn put(&mut self, x: A::Item) {
        self.data.push(x);
    }
    fn count(&self) -> usize {
        self.data.len()
    }
}
fn fill<S: Sink<u8>>(s: &mut S) {
    s.put(1);
    s.put(2);
}
fn generic_fill<A: Array>(s: &mut SmallVec<A>, x: A::Item) {
    s.put(x);
}
fn main() {
    let mut v: SmallVec<[u8; 4]> = SmallVec { data: Vec::new() };
    fill(&mut v);
    v.put(250);
    generic_fill(&mut v, 9);
    let mut w: SmallVec<[i64; 2]> = SmallVec { data: Vec::new() };
    w.put(-5);
    generic_fill(&mut w, 6);
    println!("{} {} {} {}", v.count(), v.data[2].wrapping_add(10), w.count(), w.data[0] * w.data[1]);
}
