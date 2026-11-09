// Derives over field types whose trait impls are written generic
// impls: the derive glue calls the impl's method at the concrete
// type, for `Clone`, `Default`, and `PartialOrd` as for
// `PartialEq`.
struct M<T> {
    v: Vec<T>,
}
impl<T: Clone> Clone for M<T> {
    fn clone(&self) -> M<T> {
        M { v: self.v.clone() }
    }
}
impl<T> Default for M<T> {
    fn default() -> M<T> {
        M { v: Vec::new() }
    }
}
impl<T: PartialEq> PartialEq for M<T> {
    fn eq(&self, o: &M<T>) -> bool {
        self.v.len() == o.v.len()
    }
}
impl<T: PartialOrd> PartialOrd for M<T> {
    fn partial_cmp(&self, o: &M<T>) -> Option<std::cmp::Ordering> {
        self.v.len().partial_cmp(&o.v.len())
    }
}
#[derive(Clone, Default, PartialEq, PartialOrd)]
struct W {
    m: M<String>,
    n: u32,
}
fn main() {
    let a = W { m: M { v: vec!["x".to_string()] }, n: 1 };
    let b = a.clone();
    let c = W { m: M { v: vec!["y".to_string(), "z".to_string()] }, n: 1 };
    println!("{} {} {} {:?} {}", a == b, a < c, c > a, b.m.v, W::default().n);
}
