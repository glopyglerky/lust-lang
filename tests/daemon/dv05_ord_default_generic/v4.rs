#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct P {
    a: u8,
    b: i16,
}
fn make<T: Default>() -> T {
    T::default()
}
fn main() {
    let mut v = vec![(P { a: 2, b: 1 }, 1u8), (P { a: 1, b: 9 }, 2u8)];
    v.sort();
    let d: P = make();
    let m = v.iter().map(|x| &x.0).max();
    println!("{:?} {:?} {:?}", v, d, m);
}
