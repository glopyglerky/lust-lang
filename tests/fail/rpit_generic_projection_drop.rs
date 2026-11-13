// expect: L0312
// rustc E0506: the hidden type of a generic `impl Trait` fn can be a
// projection (`I::IntoIter`); at `I = &Vec<u32>` it is `slice::Iter`,
// and the local that holds it still drops at scope end, which keeps
// the captured borrow live at the assignment.
struct C {
    v: Vec<u32>,
    n: u8,
}
fn items<I: IntoIterator>(i: I) -> impl Iterator<Item = I::Item> {
    i.into_iter()
}
fn main() {
    let mut c = C { v: vec![1, 2], n: 0 };
    let r = &c;
    let _it = items(&r.v);
    c.n = 1;
    println!("{}", c.n);
}
