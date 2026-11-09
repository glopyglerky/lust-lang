use std::rc::Rc;
fn main() { let mut v: Vec<Rc<str>> = Vec::new(); for s in ["b", "a", "b", "c"] { v.push(Rc::from(s)); } v.sort(); v.dedup(); println!("{:?}", v); let joined: Vec<&str> = v.iter().map(|s| &**s).collect(); println!("{:?}", joined); let bs: Vec<Box<str>> = v.iter().map(|s| Box::<str>::from(&**s)).collect(); println!("{:?}", bs); }
