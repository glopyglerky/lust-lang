use std::fmt::Debug;
struct Store<T> { v: Vec<T>, names: Vec<String> }
impl<T: Clone + Debug> Store<T> {
    fn names(&self) -> impl Iterator<Item = &str> + '_ { self.names.iter().map(|s| s.as_str()) }
    fn first(&self) -> Option<T> { self.v.first().cloned() }
}
fn width(s: &str, pad: usize) -> usize where u32: Copy { s.len() + pad }
fn total(st: &Store<u8>) -> usize { st.names().map(|n| width(n, 1)).sum() }
fn shown(st: &Store<u8>) -> String { format!("{:?} {}", st.first(), st.names().count() + width("x", 0)) }
fn main() {
    let st = Store { v: vec![1u8], names: vec!["ab".into(), "c".into()] };
    println!("{} {}", total(&st), shown(&st));
}
