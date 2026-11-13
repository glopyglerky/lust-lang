// Recursive enums with an `Arc<[Self]>` field, held as Option<Arc<[E]>>
// in another struct. The C slice struct holds only `E*`, so it must not
// count as needing E's definition first: that false cycle made the
// type order depend on ADT ids (and so on the file name), and tcc then
// saw a field of incomplete type. Several pairs, declared in both
// orders, so some pair always starts the walk at the slice.
use std::sync::Arc;

struct H1 {
    calls: Option<Arc<[A]>>,
}
enum A {
    Leaf(String, Option<u128>),
    P(Arc<[A]>),
    S(Arc<A>),
}

enum B {
    Leaf(u8),
    P(Arc<[B]>),
}
struct H2 {
    calls: Option<Arc<[B]>>,
}

struct H3 {
    calls: Option<Arc<[C]>>,
    more: Vec<Arc<[C]>>,
}
enum C {
    Leaf(String),
    P(Arc<[C]>),
}

enum D {
    Leaf(u32),
    P(Arc<[D]>),
    Q(Option<Arc<[D]>>),
}
struct H4 {
    d: Option<Arc<[D]>>,
}

fn sum_a(a: &[A]) -> usize {
    a.iter()
        .map(|x| match x {
            A::Leaf(s, h) => s.len() + h.unwrap_or(0) as usize,
            A::P(g) => sum_a(g),
            A::S(s) => sum_a(std::slice::from_ref(&**s)),
        })
        .sum()
}

fn sum_b(b: &[B]) -> usize {
    b.iter().map(|x| match x { B::Leaf(n) => *n as usize, B::P(g) => sum_b(g) }).sum()
}

fn sum_c(c: &[C]) -> usize {
    c.iter().map(|x| match x { C::Leaf(s) => s.len(), C::P(g) => sum_c(g) }).sum()
}

fn sum_d(d: &[D]) -> usize {
    d.iter()
        .map(|x| match x {
            D::Leaf(n) => *n as usize,
            D::P(g) => sum_d(g),
            D::Q(o) => o.as_ref().map_or(0, |g| sum_d(g)),
        })
        .sum()
}

fn main() {
    let a: Arc<[A]> = vec![A::Leaf("ab".into(), Some(2)), A::P(vec![A::Leaf("c".into(), None)].into()), A::S(Arc::new(A::Leaf("d".into(), None)))].into();
    let h1 = H1 { calls: Some(a) };
    let h2 = H2 { calls: Some(vec![B::Leaf(4), B::P(vec![B::Leaf(5)].into())].into()) };
    let c: Arc<[C]> = vec![C::Leaf("xyz".into()), C::P(vec![C::Leaf("w".into())].into())].into();
    let h3 = H3 { calls: Some(c.clone()), more: vec![c] };
    let h4 = H4 { d: Some(vec![D::Leaf(1), D::Q(Some(vec![D::Leaf(6)].into())), D::P(vec![].into())].into()) };
    println!("{}", h1.calls.as_ref().map_or(0, |g| sum_a(g)));
    println!("{}", h2.calls.as_ref().map_or(0, |g| sum_b(g)));
    println!("{} {}", h3.calls.as_ref().map_or(0, |g| sum_c(g)), h3.more.len());
    println!("{}", h4.d.as_ref().map_or(0, |g| sum_d(g)));
}
