// expect: L0312
// rustc E0506: an unused `let` binding of an opaque type still drops
// at scope end, which keeps its captured borrow live at the
// assignment.
struct C<K> {
    v: Vec<K>,
    n: u8,
}
impl<K> C<K> {
    fn items(&self) -> impl Iterator<Item = &K> {
        self.v.iter()
    }
}
fn main() {
    let mut c = C { v: vec![1u32, 2], n: 0 };
    let _it = c.items();
    c.n = 1;
    println!("{}", c.n);
}
