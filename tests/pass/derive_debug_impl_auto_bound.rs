// Written impls whose where clauses name auto traits (`S: Send`,
// `S: Sync`): the solver proves them on the matched instance, with
// Self read through the impl probe's variable.
use std::fmt;
#[derive(Clone, Default)]
struct MyBuild;
#[allow(dead_code)]
struct M<K, S> {
    k: Vec<K>,
    s: S,
}
impl<K, S> fmt::Debug for M<K, S>
where
    K: fmt::Debug,
    S: Send,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "M{:?}", self.k)
    }
}
impl<K: Clone, S: Clone + Sync> Clone for M<K, S> {
    fn clone(&self) -> Self {
        M { k: self.k.clone(), s: self.s.clone() }
    }
}
#[derive(Debug, Clone)]
struct Outer {
    m: M<u8, MyBuild>,
    n: Option<M<String, (u8, MyBuild)>>,
}
fn main() {
    let o = Outer { m: M { k: vec![1], s: MyBuild }, n: Some(M { k: vec!["a".to_string()], s: (2, MyBuild) }) };
    println!("{:?}", o.clone());
}
