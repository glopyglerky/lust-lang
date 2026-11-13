// Derived impls of generic Adts that rustc checks at the definition and
// accepts: fields of other generic types with derived or written impls,
// Vec/Option/Box/Rc of them, tuples, arrays, references, PhantomData, fn
// pointers, and paths that start at a type param (`T::Item`, which the
// derive bounds too). Some are used, some only defined.
use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::rc::Rc;
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
struct Inner<T>(T);
struct Written<T>(Option<T>);
impl<T> fmt::Debug for Written<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "W{}", self.0.is_some())
    }
}
impl<T: Clone> Clone for Written<T> {
    fn clone(&self) -> Self {
        Written(self.0.clone())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Outer<'a, T, U, const N: usize> {
    a: Inner<T>,
    b: Vec<Inner<U>>,
    c: Option<Box<Inner<T>>>,
    d: Rc<(Inner<T>, u8)>,
    e: [Inner<U>; N],
    f: Option<&'a Inner<T>>,
}
#[derive(Debug, Clone)]
enum Uses<T> {
    W(Written<T>),
    V(Vec<Written<T>>),
    E,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
struct Items<T: Iterator> {
    x: Vec<T::Item>,
    y: Option<(T::Item, u8)>,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
struct Cnt(u8);
impl Iterator for Cnt {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        None
    }
}
#[derive(Clone, Copy)]
struct CopyPair<T> {
    a: Inner<T>,
    b: (T, &'static str),
    g: PhantomData<Inner<T>>,
    h: Option<fn(T) -> T>,
}
#[derive(Debug)]
struct Unused<T> {
    w: Written<T>,
}
fn main() {
    let i = Inner(4u8);
    let o: Outer<'_, u8, u16, 2> = Outer {
        a: Inner(1),
        b: vec![Inner(2)],
        c: Some(Box::new(Inner(3))),
        d: Rc::new((Inner(5), 6)),
        e: [Inner(7), Inner(8)],
        f: Some(&i),
    };
    let o2 = o.clone();
    println!("{:?}", o.a);
    println!("{} {:?}", o == o2, o.cmp(&o2));
    let mut h = DefaultHasher::new();
    o.e.hash(&mut h);
    println!("{}", h.finish());
    let u: Uses<String> = Uses::V(vec![Written(None)]);
    println!("{:?} {:?}", u.clone(), Uses::<u8>::E);
    let it: Items<Cnt> = Items { x: vec![1, 2], y: Some((3, 4)) };
    let it2 = it.clone();
    println!("{} {} {}", it == it2, it.x.len(), it2.y.unwrap().1);
    let cp = CopyPair { a: Inner(1u8), b: (2, "x"), g: PhantomData, h: None };
    let cp2 = cp;
    println!("{} {}", cp.a.0, cp2.b.1);
    let un: Unused<u8> = Unused { w: Written(Some(1)) };
    println!("{:?}", un);
}
