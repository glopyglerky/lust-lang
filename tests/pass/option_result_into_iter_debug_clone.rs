// core 1.99 Debug of option::IntoIter (`IntoIter { inner: Item { opt } }`)
// and result::IntoIter (`IntoIter { inner }`), and their Clone, over
// non-Copy elements, in derives, generic bounds, and pretty format.
use std::fmt::Debug;
#[derive(Debug, Clone)]
struct O { a: std::option::IntoIter<String>, b: std::result::IntoIter<Vec<u8>>, n: u8 }
#[derive(Debug, Clone)]
struct G<T> { it: std::option::IntoIter<T> }
fn need<X: Clone + Debug>(x: &X) -> X { x.clone() }
fn main() {
    let mut it = Some(String::from("x")).into_iter();
    println!("{:?}", it);
    let c = it.clone();
    it.next();
    println!("{:?} {:?} {:#?}", it, c, c);
    let r = Ok::<u8, String>(4).into_iter();
    println!("{:?} {:?}", r, Err::<u8, String>("e".into()).into_iter());
    let o = O { a: c, b: Ok::<Vec<u8>, ()>(vec![1]).into_iter(), n: 3 };
    let o2 = o.clone();
    drop(o);
    println!("{:?}", o2);
    println!("{:#?}", o2);
    let g = G { it: Some(vec![String::from("v")]).into_iter() };
    println!("{:?} {:?}", need(&g), need(&g.it));
    println!("[{:>6?}]", Some(7u8).into_iter());
    let all: Vec<String> = o2.a.collect();
    println!("{:?} {}", all, o2.n);
    let mut x = None::<Box<u8>>.into_iter();
    println!("{:?} {:?}", x.clone(), x.next());
}
