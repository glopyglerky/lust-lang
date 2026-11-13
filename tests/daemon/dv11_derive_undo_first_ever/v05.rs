// A rejected derive edit keeps the kept program (its delta is undone):
// the revert takes the delta, also after an unrelated edit and a second
// reject. Traits the program never derived (Hash, Eq, PartialEq) come in,
// go, and come back. Rep's Iterator impl is bounded by Copy.
const _ROUTE: u8 = 0;
#[derive(Clone, Copy, Debug)]
struct A {
    x: u8,
    y: u8,
}
struct Rep<T>(T);
impl<T: Copy> Iterator for Rep<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        Some(self.0)
    }
}
fn main() {
    let a = A { x: 3, y: 4 };
    let b = a;
    let mut r = Rep(5u8);
    println!("{} {} {:?} {:?}", "two", a.x + b.y, a, r.next());
}
