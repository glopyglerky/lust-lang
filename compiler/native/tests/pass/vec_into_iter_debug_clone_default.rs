// vec::IntoIter Debug (`IntoIter([..])` of what it still owns), Clone (a
// fresh buffer of exactly that part), and Default (empty), as alloc 1.99.
use std::vec::IntoIter;
#[derive(Debug, Clone, Default)]
struct O { it: IntoIter<String>, n: u8 }
#[derive(Debug, Clone)]
struct Z;
#[derive(Debug, Clone)]
struct G<T> { it: IntoIter<T> }
fn need<X: Clone + std::fmt::Debug>(x: &X) -> X { x.clone() }
fn main() {
    let mut it = vec![String::from("a"), String::from("b"), String::from("c")].into_iter();
    println!("{:?}", it);
    it.next();
    let c = it.clone();
    println!("{:?} {:?} {:#?}", it, c, c);
    println!("[{:>5?}]", vec![1u8, 2].into_iter());
    let o = O { it: c, n: 1 };
    let o2 = o.clone();
    drop(o);
    println!("{:?}", o2);
    let d = O::default();
    println!("{:?} {}", d, d.n);
    let z = vec![Z, Z].into_iter();
    println!("{:?} {:?}", z.clone(), need(&z));
    let mut g = G { it: vec![vec![1u16], vec![]].into_iter() };
    g.it.next_back();
    let g2 = g.clone();
    println!("{:?} {:?} {}", g, g2, g2.it.len());
    let e: IntoIter<u8> = Default::default();
    println!("{:?} {}", e, e.len());
    let all: Vec<String> = o2.it.collect();
    println!("{:?}", all);
}
