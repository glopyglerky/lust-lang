// Derived Hash, PartialEq, and Eq on recursive types through
// Option<Box<..>>, Vec, Rc, and Arc: rustc proves the field goals
// from the derive impl itself (a coinductive cycle), so a field of
// the deriving type must not fail the derive bound check.
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::Arc;
#[derive(Hash, PartialEq, Eq, Debug)]
struct S { n: u32, parent: Option<Box<S>> }
#[derive(Hash, PartialEq, Eq, Debug)]
struct T { kids: Vec<T>, rc: Option<Rc<T>>, arc: Option<Arc<T>> }
#[derive(Hash, PartialEq, Eq, Debug)]
struct A { b: Option<Box<B>> }
#[derive(Hash, PartialEq, Eq, Debug)]
struct B { a: Option<Box<A>>, n: u8 }
#[derive(Hash, PartialEq, Eq)]
struct F { p: Option<Box<F>>, f: fn(u8) -> u8 }
fn inc(x: u8) -> u8 { x + 1 }
fn main() {
    let s = S { n: 1, parent: Some(Box::new(S { n: 2, parent: None })) };
    let mut set = HashSet::new();
    set.insert(s);
    let probe = S { n: 1, parent: Some(Box::new(S { n: 2, parent: None })) };
    let other = S { n: 3, parent: None };
    println!("{} {}", set.contains(&probe), set.contains(&other));
    let t = T { kids: vec![T { kids: vec![], rc: None, arc: None }], rc: None, arc: None };
    let mut set2 = HashSet::new();
    set2.insert(t);
    println!("{}", set2.contains(&T { kids: vec![T { kids: vec![], rc: None, arc: None }], rc: None, arc: None }));
    let a = A { b: Some(Box::new(B { a: None, n: 7 })) };
    println!("{:?}", a);
    let f1 = F { p: Some(Box::new(F { p: None, f: inc })), f: inc };
    let f2 = F { p: Some(Box::new(F { p: None, f: inc })), f: inc };
    println!("{}", f1 == f2);
}
