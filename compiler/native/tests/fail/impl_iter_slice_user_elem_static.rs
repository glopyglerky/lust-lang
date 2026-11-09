// expect: L0733
// rustc: lifetime may not live long enough (a slice iterator over a user
// element type yields borrows of `self`).
struct N {
    x: u8,
}
struct S {
    v: Vec<N>,
}
impl S {
    fn names(&self) -> impl Iterator<Item = &'static N> + '_ {
        self.v.iter()
    }
}
fn main() {
    let s = S { v: vec![N { x: 1 }] };
    for n in s.names() {
        println!("{}", n.x);
    }
}
