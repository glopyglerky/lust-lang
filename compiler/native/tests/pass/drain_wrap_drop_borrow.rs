// A borrowing value that is moved out (here into `drop`) stops
// borrowing its source at the move, even when its type's drop may
// touch the borrowed data (a `vec::Drain` over a `Drop` element in
// a user wrapper, the SmallVec shape). The scope-end `Drop` of the
// moved-from local is dead, and so is its copy on the unwind path
// from later calls.
struct D(u8);
impl Drop for D {
    fn drop(&mut self) {
        print!("d{} ", self.0);
    }
}
struct MyVec(Vec<D>);
struct MyDrain<'a> {
    inner: std::vec::Drain<'a, D>,
}
impl Iterator for MyDrain<'_> {
    type Item = D;
    fn next(&mut self) -> Option<D> {
        self.inner.next()
    }
}
impl MyVec {
    fn drain(&mut self) -> MyDrain<'_> {
        MyDrain { inner: self.0.drain(1..4) }
    }
    fn capacity(&self) -> usize {
        self.0.capacity()
    }
}
impl core::ops::Deref for MyVec {
    type Target = [D];
    fn deref(&self) -> &[D] {
        &self.0
    }
}
fn main() {
    let mut y = MyVec(vec![D(0), D(1), D(2), D(3), D(4)]);
    let mut d = y.drain();
    let first = d.next();
    drop(d);
    drop(first);
    println!("{} {}", y.len(), y.capacity());
    drop(y);
    println!("done");
}
