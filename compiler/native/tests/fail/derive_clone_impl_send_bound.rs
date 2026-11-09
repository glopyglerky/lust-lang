// expect: L0112
// rustc E0277: the written `Clone` needs `S: Send`, and `Rc<u8>` is not `Send`.
use std::rc::Rc;
#[allow(dead_code)]
struct M<K, S> { k: Vec<K>, s: S }
impl<K: Clone, S: Clone> Clone for M<K, S> where S: Send {
    fn clone(&self) -> Self { M { k: self.k.clone(), s: self.s.clone() } }
}
#[derive(Clone)]
struct Outer { m: M<u8, Rc<u8>> }
fn main() { let o = Outer { m: M { k: vec![1], s: Rc::new(1) } }; let c = o.clone(); println!("{}", c.m.k.len()); }
