use std::collections::{BTreeSet, HashSet};
use std::rc::Rc;
fn main() {
    let mut v: Vec<Rc<str>> = vec![Rc::from("b"), Rc::from("a"), Rc::from("c")];
    v.sort();
    println!("{:?}", v);
    let h: HashSet<Box<str>> = ["x", "y", "x"].iter().map(|s| -> Box<str> { Box::from(*s) }).collect();
    println!("{}", h.len());
    let a: Rc<[i32]> = Rc::from(vec![1, 2]); let b: Rc<[i32]> = Rc::from(vec![1, 3]);
    println!("{} {:?}", a < b, a.partial_cmp(&b));
}
