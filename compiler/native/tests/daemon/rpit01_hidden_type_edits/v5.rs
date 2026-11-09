// impl Trait callers on one warm daemon: the definer's body changes its
// hidden type (the callers' text does not), the hidden type moves to a
// generic definer, and an iterator definer changes its hidden iterator.
trait Shape {
    fn area(&self) -> u32;
}
struct Sq(u32);
struct Rect(u32, u32);
impl Shape for Sq {
    fn area(&self) -> u32 {
        self.0 * self.0
    }
}
impl Shape for Rect {
    fn area(&self) -> u32 {
        self.0 + self.1
    }
}
fn wrap<S: Shape>(s: S) -> impl Shape {
    s
}
fn make(n: u32) -> impl Shape {
    wrap(Rect(n + 1, n))
}
fn nums(n: u32) -> impl Iterator<Item = u32> {
    vec![n; n as usize].into_iter()
}
fn total(n: u32) -> u32 {
    let mut t = make(n).area();
    for i in nums(n) {
        t += i;
    }
    t
}
fn main() {
    println!("{}", total(3));
    println!("{}", make(4).area());
}
