// A derived impl of a generic Adt applied at a concrete arg inside
// another derive: the derive's bounds on the arg decide (`T: Debug`
// for `CowChunks<Table>`), a written impl with no bound holds at any
// arg, and std's PhantomData impls have no bounds.
use std::fmt;
use std::marker::PhantomData;
#[derive(Debug, Clone, PartialEq)]
enum Chunk<T: 'static> {
    Own(T),
    Freed,
}
#[derive(Debug, Clone, PartialEq)]
struct CowChunks<T: 'static> {
    chunks: Vec<Chunk<T>>,
    tail: Vec<T>,
}
#[derive(Debug, Clone, PartialEq, Default)]
struct Table {
    n: u8,
}
struct NoTraits;
struct Loose<T>(Option<fn() -> T>, u8);
impl<T> fmt::Debug for Loose<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Loose{}", self.1)
    }
}
#[derive(Debug, Clone, PartialEq)]
struct Arena {
    tables: CowChunks<Table>,
    boxed: Option<Box<CowChunks<Vec<Table>>>>,
}
#[derive(Clone, PartialEq)]
struct Marks {
    m: CowChunks<PhantomData<NoTraits>>,
}
#[derive(Debug)]
struct Holder<U> {
    loose: Loose<NoTraits>,
    u: U,
}
fn main() {
    let a = Arena {
        tables: CowChunks { chunks: vec![Chunk::Own(Table { n: 1 }), Chunk::Freed], tail: vec![Table::default()] },
        boxed: Some(Box::new(CowChunks { chunks: Vec::new(), tail: vec![vec![Table { n: 7 }]] })),
    };
    let b = a.clone();
    println!("{:?}", a);
    println!("{}", a == b);
    let h = Holder { loose: Loose(None, 5), u: 'x' };
    println!("{:?} {}", h, h.loose.0.is_none());
    let m = Marks { m: CowChunks { chunks: vec![Chunk::Freed], tail: vec![PhantomData] } };
    println!("{}", m.clone() == m);
}
