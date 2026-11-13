// `peek` borrows the Peekable only for the returned reference; a peeked
// `Ref` guard does not keep the iterator borrowed until it drops (R12).
use std::cell::{Ref, RefCell};
fn a(tasks: &[RefCell<u32>]) -> bool {
    let mut files = tasks.iter().map(|t| t.borrow()).peekable();
    files.peek().is_some() && files.all(|t| *t > 0)
}
fn d<'a>(t: &'a [RefCell<u32>]) -> bool {
    let v: Vec<Ref<'a, u32>> = t.iter().map(|t| t.borrow()).collect();
    let mut f = v.into_iter().peekable();
    f.peek().is_some()
}
fn e(src: &[u32]) -> u32 { let mut f = src.iter().peekable(); let mut n = 0; while let Some(p) = f.peek() { n += **p; f.next(); } n }
fn main() { let v = [RefCell::new(1)]; println!("{} {} {}", a(&v), d(&v), e(&[1, 2])); }
