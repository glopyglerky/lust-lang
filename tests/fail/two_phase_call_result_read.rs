// expect: L0303
// rustc E0503: the receiver is a `&mut` a call returned; its loan of `a`
// is not two-phase, so reading `a.n` in the argument conflicts.
struct Acc { n: i32, items: Vec<i32> }
impl Acc {
    fn items_mut(&mut self) -> &mut Vec<i32> { &mut self.items }
}
fn main() {
    let mut a = Acc { n: 1, items: vec![] };
    a.items_mut().push(a.n);
    println!("{:?}", a.items);
}
