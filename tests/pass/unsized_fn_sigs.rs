use std::rc::Rc; use std::sync::Arc;
fn mk(n: u32) -> Rc<str> { Rc::from(format!("v{n}")) }
fn len_of<T: AsRef<str>>(t: T) -> usize { t.as_ref().len() }
fn sum(xs: Arc<[u64]>) -> u64 { xs.iter().sum() }
fn first(b: &Box<str>) -> Option<char> { b.chars().next() }
fn main() { let r = mk(7); println!("{} {} {}", r, len_of(r.clone()), sum(Arc::from(vec![1, 2, 3]))); let b: Box<str> = "q".into(); println!("{:?}", first(&b)); }
