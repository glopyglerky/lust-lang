use std::rc::Rc;
fn takes(s: &str) -> usize { s.len() }
fn takes_sl(s: &[u8]) -> usize { s.len() }
fn main() { let b: Box<str> = "abc".into(); let r: Rc<[u8]> = Rc::from(vec![1u8, 2]); let a: std::sync::Arc<str> = "zz".into(); println!("{} {} {}", takes(&b), takes_sl(&r), takes(&a)); }
