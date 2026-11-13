// Derived Clone, PartialEq, PartialOrd, Debug, and Default on recursive
// generic types (through Option<Box<..>> and Vec<..>).
#[derive(Clone, PartialEq, Debug)]
struct L<T> { v: T, next: Option<Box<L<T>>> }
fn list() {
    let a = L { v: 1u8, next: Some(Box::new(L { v: 2, next: None })) };
    let b = a.clone();
    println!("{} {:?}", a == b, b);
}
#[derive(Clone, PartialEq, PartialOrd, Debug, Default)]
struct T2<X> { v: X, kids: Vec<T2<X>> }
fn tree() {
    let a = T2 { v: 1u8, kids: vec![T2 { v: 2, kids: vec![] }] };
    let b = a.clone();
    let d: T2<u16> = T2::default();
    println!("{} {} {:?} {:?}", a == b, a < b, b, d);
}
fn main() {
    list();
    tree();
}
