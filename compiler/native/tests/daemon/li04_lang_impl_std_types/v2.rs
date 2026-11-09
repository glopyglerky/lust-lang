use std::cmp::Ordering;

struct Name(&'static str);
impl From<Name> for String {
    fn from(n: Name) -> String {
        format!("[{}]", n.0)
    }
}
fn mk() -> String {
    let s: String = Name("a").into();
    s
}
#[derive(Debug)]
struct C {
    k: i32,
}
impl Default for C {
    fn default() -> C {
        C { k: 9 }
    }
}
fn dflt() -> C {
    C::default()
}
#[derive(Debug, PartialEq, Eq)]
struct K(i32);
impl PartialOrd for K {
    fn partial_cmp(&self, o: &K) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}
impl Ord for K {
    fn cmp(&self, o: &K) -> Ordering {
        self.0.cmp(&o.0)
    }
}
fn order() -> (bool, bool) {
    (K(1) < K(2), K(3).cmp(&K(2)) == Ordering::Greater)
}
fn main() {
    println!("{}", mk());
    println!("{:?}", dflt());
    println!("{:?}", order());
}
