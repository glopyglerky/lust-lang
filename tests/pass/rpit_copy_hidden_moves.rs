// An `impl Trait` whose hidden type is `Copy` while its bounds are not:
// rustc moves the value, so a value moved before a write no longer
// holds its borrow, and each value is used once.
struct C {
    v: Vec<u32>,
    n: u8,
}
fn consume<T>(_t: T) {}
fn len_of(v: &Vec<u32>) -> impl Sized + std::fmt::Debug {
    v.len()
}
fn reader(c: &C) -> impl FnOnce() -> u8 + '_ {
    move || c.n + 1
}
fn shown(c: &C) -> impl std::fmt::Display + '_ {
    &c.n
}
fn main() {
    let mut c = C { v: vec![1, 2, 3], n: 4 };
    let l = len_of(&c.v);
    println!("{:?}", l);
    consume(l);
    c.v.push(9);
    let f = reader(&c);
    println!("{}", f());
    c.n = 5;
    let o = Some(shown(&c));
    let s = o.unwrap();
    println!("{}", s);
    drop(s);
    c.n = 6;
    println!("{} {}", c.n, c.v.len());
}
