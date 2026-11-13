use std::rc::Rc; use std::sync::Arc;
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
struct K { name: Box<str>, tag: Rc<str>, path: Arc<str>, bytes: Rc<[u8]> }
fn main() {
    let k = K { name: "n".into(), tag: Rc::from("t"), path: Arc::from("p"), bytes: Rc::from(vec![1u8]) };
    let k2 = k.clone();
    println!("{:?} {} {:?}", k, k == k2, k.cmp(&K::default()));
    let mut s = std::collections::HashSet::new(); s.insert(k.clone()); s.insert(k2); println!("{}", s.len());
}
