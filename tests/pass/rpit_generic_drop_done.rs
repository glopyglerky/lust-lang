// A generic `impl Trait` value that is dropped or consumed before an
// assignment to its borrow source: rustc accepts each form, and the
// drop-live treatment of opaque values must not reject them.
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
    let it = c.items();
    drop(it);
    c.n = 1;
    for x in c.items() {
        println!("{}", x);
    }
    c.n = 2;
    let k = c.items().count();
    c.n = 3;
    {
        let mut it = c.items();
        let _ = it.next();
    }
    c.n = 4;
    println!("{} {}", k, c.n);
}
