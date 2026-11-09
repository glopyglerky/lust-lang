// A flat_map closure calling a `&mut` method through a captured `RefMut`
// borrows the guard only while the iterator lives (R12).
use std::cell::RefCell;
struct S { m: Vec<u32> }
impl S { fn get(&mut self, i: usize) -> Vec<u32> { vec![self.m[i]] } }
fn a(c: &RefCell<S>, t: &[usize]) -> Vec<u32> { let mut s = c.borrow_mut(); let d: Vec<u32> = t.iter().flat_map(|&i| s.get(i)).collect(); drop(s); d }
fn f(c: &RefCell<S>, t: &[usize]) -> usize { let mut s = c.borrow_mut(); let d = t.iter().flat_map(|&i| s.get(i)).count(); drop(s); d }
fn main() { let c = RefCell::new(S { m: vec![2] }); println!("{:?} {}", a(&c, &[0]), f(&c, &[0])); }
