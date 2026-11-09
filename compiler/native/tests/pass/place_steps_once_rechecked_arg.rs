// An argument that check looks at more than once (to pick a `from` form,
// a trait input, or a builtin conversion) runs its user `Deref` and
// `Index` steps once.
use std::convert::TryFrom;
use std::ops::{Deref, Index};
use std::rc::Rc;

struct W(String);
impl Deref for W {
    type Target = String;
    fn deref(&self) -> &String {
        println!("deref W");
        &self.0
    }
}
struct V(Vec<u8>);
impl Deref for V {
    type Target = Vec<u8>;
    fn deref(&self) -> &Vec<u8> {
        println!("deref V");
        &self.0
    }
}
struct T(Vec<String>);
impl Index<usize> for T {
    type Output = String;
    fn index(&self, i: usize) -> &String {
        println!("index {i}");
        &self.0[i]
    }
}
#[derive(Debug)]
struct M(usize);
impl From<&String> for M {
    fn from(s: &String) -> M {
        M(s.len())
    }
}
#[derive(Debug)]
struct N(usize);
impl TryFrom<&String> for N {
    type Error = ();
    fn try_from(s: &String) -> Result<N, ()> {
        Ok(N(s.len()))
    }
}
trait Conv<X> {
    fn conv(&self, x: X) -> usize;
}
struct S;
impl Conv<&String> for S {
    fn conv(&self, x: &String) -> usize {
        x.len()
    }
}
impl Conv<u8> for S {
    fn conv(&self, x: u8) -> usize {
        x as usize
    }
}

fn main() {
    let w = W(String::from("abc"));
    let v = V(vec![1, 2]);
    let t = T(vec![String::from("p"), String::from("qq")]);
    println!("{}", String::from(&**w));
    println!("{:?}", Vec::from(&v[..]));
    let r: Rc<str> = Rc::from(&**w);
    let b: Box<[u8]> = Box::from(&**v);
    println!("{} {:?}", r, b);
    println!("{:?} {:?}", M::from(&*w), M::from(&t[1]));
    println!("{:?}", N::try_from(&*w));
    println!("{} {}", S.conv(&*w), S.conv(&t[0]));
}
