// ARCH 2.5: projections normalize at every use, as rustc does. The
// smallvec adapter shape: `SmallVec<A: Array>` with `A::Item` in
// params, returns, fields, struct literals, and an impl header's
// trait args (`impl<A: Array> Sink<A::Item> for SmallVec<A>`).
pub unsafe trait Array {
    type Item;
    fn size() -> usize;
}
macro_rules! impl_array {
    ($($n:expr),*) => { $(unsafe impl<T> Array for [T; $n] { type Item = T; fn size() -> usize { $n } })* }
}
impl_array!(2, 4, 8);
pub struct SmallVec<A: Array> {
    cap: usize,
    data: Vec<A::Item>,
}
impl<A: Array> SmallVec<A> {
    pub fn new() -> Self {
        SmallVec { cap: A::size(), data: Vec::new() }
    }
    pub fn push(&mut self, value: A::Item) {
        self.data.push(value);
    }
    pub fn pop(&mut self) -> Option<A::Item> {
        self.data.pop()
    }
    pub fn spilled(&self) -> bool {
        self.data.len() > self.cap
    }
}
pub trait Sink<X> {
    fn put(&mut self, x: X);
}
impl<A: Array> Sink<A::Item> for SmallVec<A> {
    fn put(&mut self, x: A::Item) {
        self.data.push(x);
    }
}
fn fill<S: Sink<u8>>(s: &mut S) {
    s.put(1);
    s.put(2);
}
fn main() {
    let mut v: SmallVec<[u8; 2]> = SmallVec::new();
    v.push(250);
    fill(&mut v);
    let top = v.pop().unwrap();
    println!("{} {} {}", top, v.spilled(), v.data[0].wrapping_add(10));
    let mut w = SmallVec::<[u16; 4]>::new();
    w.push(40000);
    println!("{} {}", w.data[0] + 1, w.spilled());
}
