use std::cmp::Ordering;

struct Name(&'static str);
fn mk() -> String {
    let s: String = Name("a").into();
    s
}
#[derive(Debug, Default)]
struct C {
    k: i32,
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
        o.0.cmp(&self.0)
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
